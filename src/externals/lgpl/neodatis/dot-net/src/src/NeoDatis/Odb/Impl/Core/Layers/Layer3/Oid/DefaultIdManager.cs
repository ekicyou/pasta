namespace NeoDatis.Odb.Impl.Core.Layers.Layer3.Oid
{
	/// <summary>Class to manage the ids of all the objects of the database.</summary>
	/// <remarks>Class to manage the ids of all the objects of the database.</remarks>
	/// <author>osmadja</author>
	public class DefaultIdManager : NeoDatis.Odb.Core.Layers.Layer3.IIdManager
	{
		private static readonly string LogId = "IdManager";

		private NeoDatis.Odb.Core.ICoreProvider provider;

		private long currentBlockIdPosition;

		private int currentBlockIdNumber;

		public NeoDatis.Odb.OID nextId;

		public NeoDatis.Odb.OID maxId;

		protected NeoDatis.Odb.Core.Layers.Layer3.IObjectWriter objectWriter;

		protected NeoDatis.Odb.Core.Layers.Layer3.IObjectReader objectReader;

		protected NeoDatis.Odb.Core.Transaction.ISession session;

		/// <summary>Contains the last ids: id value,id position, id value, id position=&gt; the array is created with twice the size
		/// 	</summary>
		protected NeoDatis.Odb.OID[] lastIds;

		protected long[] lastIdPositions;

		private int lastIdIndex;

		private const int IdBufferSize = 10;

		/// <param name="objectWriter">The object writer</param>
		/// <param name="objectReader">The object reader</param>
		/// <param name="currentBlockIdPosition">The position of the current block</param>
		/// <param name="currentBlockIdNumber">The number of the current block</param>
		/// <param name="currentMaxId">Maximum Database id</param>
		public DefaultIdManager(NeoDatis.Odb.Core.Layers.Layer3.IObjectWriter objectWriter
			, NeoDatis.Odb.Core.Layers.Layer3.IObjectReader objectReader, long currentBlockIdPosition
			, int currentBlockIdNumber, NeoDatis.Odb.OID currentMaxId)
		{
			this.provider = NeoDatis.Odb.OdbConfiguration.GetCoreProvider();
			this.objectWriter = objectWriter;
			this.objectReader = objectReader;
			this.session = objectWriter.GetSession();
			this.currentBlockIdPosition = currentBlockIdPosition;
			this.currentBlockIdNumber = currentBlockIdNumber;
			this.maxId = provider.GetObjectOID((long)currentBlockIdNumber * NeoDatis.Odb.OdbConfiguration
				.GetNB_IDS_PER_BLOCK(), 0);
			this.nextId = provider.GetObjectOID(currentMaxId.GetObjectId() + 1, 0);
			lastIds = new NeoDatis.Odb.OID[IdBufferSize];
			for (int i = 0; i < IdBufferSize; i++)
			{
				lastIds[i] = NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant.NullObjectId;
			}
			lastIdPositions = new long[IdBufferSize];
			lastIdIndex = 0;
		}

		/// <summary>To check if the id block must shift: that a new id block must be created
		/// 	</summary>
		/// <returns>a boolean value to check if block of id is full</returns>
		public virtual bool MustShift()
		{
			lock (this)
			{
				return nextId.CompareTo(maxId) > 0;
			}
		}

		/// <summary>Gets an id for an object (instance)</summary>
		/// <param name="objectPosition">the object position (instance)</param>
		/// <param name="idType">The type id : object,class, unknown</param>
		/// <param name="label">A label for debug</param>
		/// <returns>The id</returns>
		internal virtual NeoDatis.Odb.OID GetNextId(long objectPosition, byte idType, byte
			 idStatus, string label)
		{
			lock (this)
			{
				if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
				{
					NeoDatis.Tool.DLogger.Debug("  Start of " + label + " for object with position " 
						+ objectPosition);
				}
				if (MustShift())
				{
					ShiftBlock();
				}
				// Keep the current id
				NeoDatis.Odb.OID currentNextId = nextId;
				if (idType == NeoDatis.Odb.Core.Layers.Layer3.IDTypes.Class)
				{
					// If its a class, build a class OID instead.
					currentNextId = provider.GetClassOID(currentNextId.GetObjectId());
				}
				// Compute the new index to be used to store id and its position in the lastIds and lastIdPositions array
				int currentIndex = (lastIdIndex + 1) % IdBufferSize;
				// Stores the id
				lastIds[currentIndex] = currentNextId;
				// really associate id to the object position
				long idPosition = AssociateIdToObject(idType, idStatus, objectPosition);
				// Store the id position
				lastIdPositions[currentIndex] = idPosition;
				if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
				{
					NeoDatis.Tool.DLogger.Debug("  End of " + label + " for object with position " + 
						idPosition + " : returning " + currentNextId);
				}
				// Update the id buffer index
				lastIdIndex = currentIndex;
				return currentNextId;
			}
		}

		public virtual NeoDatis.Odb.OID GetNextObjectId(long objectPosition)
		{
			lock (this)
			{
				return GetNextId(objectPosition, NeoDatis.Odb.Core.Layers.Layer3.IDTypes.Object, 
					NeoDatis.Odb.Impl.Core.Layers.Layer3.Oid.IDStatus.Active, "getNextObjectId");
			}
		}

		public virtual NeoDatis.Odb.OID GetNextClassId(long objectPosition)
		{
			lock (this)
			{
				return GetNextId(objectPosition, NeoDatis.Odb.Core.Layers.Layer3.IDTypes.Class, NeoDatis.Odb.Impl.Core.Layers.Layer3.Oid.IDStatus
					.Active, "getNextClassId");
			}
		}

		public virtual void UpdateObjectPositionForOid(NeoDatis.Odb.OID oid, long objectPosition
			, bool writeInTransaction)
		{
			//TODO Remove comments here
			// Id may be negative to differ from positions
			//if(id<0){
			//	id = -id;
			//}
			long idPosition = GetIdPosition(oid);
			objectWriter.UpdateObjectPositionForObjectOIDWithPosition(idPosition, objectPosition
				, writeInTransaction);
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
			{
				NeoDatis.Tool.DLogger.Debug("IDManager : Updating id " + oid + " with position " 
					+ objectPosition);
			}
		}

		public virtual void UpdateClassPositionForId(NeoDatis.Odb.OID classId, long objectPosition
			, bool writeInTransaction)
		{
			// TODO Remove comments here
			// Id may be negative to differ from positions
			//if(classId<0){
			//    classId = -classId;
			//}
			long idPosition = GetIdPosition(classId);
			objectWriter.UpdateClassPositionForClassOIDWithPosition(idPosition, objectPosition
				, writeInTransaction);
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
			{
				NeoDatis.Tool.DLogger.Debug("Updating id " + classId + " with position " + objectPosition
					);
			}
		}

		public virtual void UpdateIdStatus(NeoDatis.Odb.OID id, byte newStatus)
		{
			long idPosition = GetIdPosition(id);
			objectWriter.UpdateStatusForIdWithPosition(idPosition, newStatus, true);
		}

		private long GetIdPosition(NeoDatis.Odb.OID oid)
		{
			// first check if it is the last
			if (lastIds[lastIdIndex] != null && lastIds[lastIdIndex].Equals(oid))
			{
				return lastIdPositions[(lastIdIndex)];
			}
			for (int i = 0; i < IdBufferSize; i++)
			{
				if (lastIds[i] != null && lastIds[i].Equals(oid))
				{
					return lastIdPositions[i];
				}
			}
			// object id is not is cache
			return objectReader.ReadOidPosition(oid);
		}

		private long AssociateIdToObject(byte idType, byte idStatus, long objectPosition)
		{
			long idPosition = objectWriter.AssociateIdToObject(idType, idStatus, currentBlockIdPosition
				, nextId, objectPosition, false);
			nextId = provider.GetObjectOID(nextId.GetObjectId() + 1, 0);
			return idPosition;
		}

		private void ShiftBlock()
		{
			long currentBlockPosition = this.currentBlockIdPosition;
			// the block has reached the end, , must create a new id block
			long newBlockPosition = CreateNewBlock();
			// Mark the current block as full
			MarkBlockAsFull(currentBlockPosition, newBlockPosition);
			this.currentBlockIdNumber++;
			this.currentBlockIdPosition = newBlockPosition;
			this.maxId = provider.GetObjectOID((long)currentBlockIdNumber * NeoDatis.Odb.OdbConfiguration
				.GetNB_IDS_PER_BLOCK(), 0);
		}

		private void MarkBlockAsFull(long currentBlockIdPosition, long nextBlockPosition)
		{
			objectWriter.MarkIdBlockAsFull(currentBlockIdPosition, nextBlockPosition, false);
		}

		private long CreateNewBlock()
		{
			long position = objectWriter.WriteIdBlock(-1, NeoDatis.Odb.OdbConfiguration.GetIdBlockSize
				(), NeoDatis.Odb.Impl.Core.Layers.Layer3.Block.BlockStatus.BlockNotFull, currentBlockIdNumber
				 + 1, currentBlockIdPosition, false);
			return position;
		}

		public virtual NeoDatis.Odb.OID ConsultNextOid()
		{
			lock (this)
			{
				return nextId;
			}
		}

		public virtual void ReserveIds(long nbIds)
		{
			NeoDatis.Odb.OID id = null;
			while (nextId.GetObjectId() < nbIds + 1)
			{
				id = GetNextId(-1, NeoDatis.Odb.Core.Layers.Layer3.IDTypes.Unknown, NeoDatis.Odb.Impl.Core.Layers.Layer3.Oid.IDStatus
					.Unknown, "reserving id");
				if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
				{
					NeoDatis.Tool.DLogger.Debug("reserving id " + id);
				}
			}
			return;
		}

		public virtual long GetObjectPositionWithOid(NeoDatis.Odb.OID oid, bool useCache)
		{
			return objectReader.GetObjectPositionFromItsOid(oid, useCache, true);
		}

		public virtual void Clear()
		{
			objectReader = null;
			objectWriter = null;
			session = null;
			lastIdPositions = null;
			lastIds = null;
		}

		protected virtual NeoDatis.Odb.Core.Transaction.ISession GetSession()
		{
			return session;
		}
	}
}
