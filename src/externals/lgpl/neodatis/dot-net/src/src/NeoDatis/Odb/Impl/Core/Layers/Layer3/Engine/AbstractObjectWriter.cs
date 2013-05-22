using NeoDatis.Odb.Core.Layers.Layer2.Meta;
namespace NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine
{
	/// <summary>Manage all IO writing</summary>
	/// <author>olivier s</author>
	public abstract class AbstractObjectWriter : NeoDatis.Odb.Core.Layers.Layer3.IObjectWriter
	{
		private static readonly int NonNativeHeaderBlockSize = NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
			.Integer.GetSize() + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Byte.GetSize()
			 + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Long.GetSize();

		private static readonly int NativeHeaderBlockSize = NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
			.Integer.GetSize() + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Byte.GetSize()
			 + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Integer.GetSize() + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
			.Boolean.GetSize();

		private static byte[] NativeHeaderBlockSizeByte = null;

		protected static int nbInPlaceUpdates = 0;

		protected static int nbNormalUpdates = 0;

		public static readonly string LogId = "ObjectWriter";

		public static readonly string LogIdDebug = "ObjectWriter.debug";

		protected NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine storageEngine;

		protected NeoDatis.Odb.Core.Layers.Layer3.IObjectReader objectReader;

		public NeoDatis.Odb.Core.Layers.Layer1.Introspector.IClassIntrospector classIntrospector;

		public NeoDatis.Odb.Core.Layers.Layer3.Engine.IFileSystemInterface fsi;

		private int currentDepth;

		protected NeoDatis.Odb.Core.Layers.Layer3.IIdManager idManager;

		/// <summary>To manage triggers</summary>
		protected NeoDatis.Odb.Core.Trigger.ITriggerManager triggerManager;

		protected NeoDatis.Odb.Core.Layers.Layer3.Engine.IByteArrayConverter byteArrayConverter;

		private static int nbCallsToUpdate;

		private bool isLocalMode;

		protected NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.IObjectInfoComparator comparator;

		public AbstractObjectWriter(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine
			)
		{
			// public ISession session;
			// Just for display matters
			this.storageEngine = engine;
			this.objectReader = storageEngine.GetObjectReader();
			this.isLocalMode = this.storageEngine.IsLocal();
			NeoDatis.Odb.Core.ICoreProvider provider = NeoDatis.Odb.OdbConfiguration.GetCoreProvider
				();
			this.byteArrayConverter = provider.GetByteArrayConverter();
			this.classIntrospector = provider.GetClassIntrospector();
			NativeHeaderBlockSizeByte = byteArrayConverter.IntToByteArray(NativeHeaderBlockSize
				);
			comparator = new NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Compare.ObjectInfoComparator
				();
		}

		public abstract NeoDatis.Odb.Core.Transaction.ISession GetSession();

		public abstract NeoDatis.Odb.Core.Layers.Layer3.Engine.IFileSystemInterface BuildFSI
			();

		/// <summary>
		/// The init2 method is the two phase init implementation The
		/// FileSystemInterface depends on the session creation which is done by
		/// subclasses after the ObjectWriter constructor So we can not execute the
		/// buildFSI in the constructor as it would result in a non initialized
		/// object reference (the session)
		/// </summary>
		public virtual void Init2()
		{
			this.fsi = BuildFSI();
		}

		public virtual void AfterInit()
		{
			this.objectReader = storageEngine.GetObjectReader();
			this.idManager = NeoDatis.Odb.OdbConfiguration.GetCoreProvider().GetClientIdManager
				(storageEngine);
		}

		/// <summary>Creates the header of the file</summary>
		/// <param name="creationDate">The creation date</param>
		/// <param name="user">The user</param>
		/// <param name="password">The password @</param>
		public virtual void CreateEmptyDatabaseHeader(long creationDate, string user, string
			 password)
		{
			WriteEncrytionFlag(false, false);
			WriteVersion(false);
			NeoDatis.Odb.DatabaseId databaseId = WriteDatabaseId(creationDate, false);
			WriteReplicationFlag(false, false);
			// Create the first Transaction Id
			NeoDatis.Odb.TransactionId tid = new NeoDatis.Odb.Impl.Core.Oid.TransactionIdImpl
				(databaseId, 0, 1);
			storageEngine.SetCurrentTransactionId(tid);
			WriteLastTransactionId(tid);
			WriteNumberOfClasses(0, false);
			WriteFirstClassInfoOID(NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.NullObjectId, false);
			WriteLastODBCloseStatus(false, false);
			WriteDatabaseCharacterEncoding(false);
			WriteUserAndPassword(user, password, false);
			// This is the position of the first block id. But it will always
			// contain the position of the current id block
			fsi.WriteLong(NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant.DatabaseHeaderFirstIdBlockPosition
				, false, "current id block position", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
				.DirectWriteAction);
			// Write an empty id block
			WriteIdBlock(-1, NeoDatis.Odb.OdbConfiguration.GetIdBlockSize(), NeoDatis.Odb.Impl.Core.Layers.Layer3.Block.BlockStatus
				.BlockNotFull, 1, -1, false);
			Flush();
			storageEngine.SetCurrentIdBlockInfos(NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.DatabaseHeaderFirstIdBlockPosition, 1, NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID
				(0));
		}

		public virtual void WriteUserAndPassword(string user, string password, bool writeInTransaction
			)
		{
			if (user != null && password != null)
			{
				string encryptedPassword = NeoDatis.Odb.Impl.Tool.Cryptographer.Encrypt(password);
				fsi.WriteBoolean(true, writeInTransaction, "has user and password");
				if (user.Length > 20)
				{
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.UserNameTooLong
						.AddParameter(user).AddParameter(20));
				}
				if (password.Length > 20)
				{
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.PasswordTooLong
						.AddParameter(20));
				}
				fsi.WriteString(user, writeInTransaction, true, 50);
				fsi.SetWritePosition(NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
					.DatabaseHeaderDatabasePassword, writeInTransaction);
				fsi.WriteString(encryptedPassword, writeInTransaction, true, 50);
			}
			else
			{
				fsi.WriteBoolean(false, writeInTransaction, "database without user and password");
				fsi.WriteString("no-user", writeInTransaction, true, 50);
				fsi.WriteString("no-password", writeInTransaction, true, 50);
			}
		}

		/// <summary>Write the encryption flag : 0= no encryption, 1=with encryption</summary>
		public virtual void WriteEncrytionFlag(bool useEncryption, bool writeInTransaction
			)
		{
			fsi.SetWritePosition(NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.DatabaseHeaderUseEncryptionPosition, writeInTransaction);
			fsi.WriteByte(useEncryption ? NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.WithEncryption : NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.NoEncryption, writeInTransaction, "encryption flag");
		}

		/// <summary>Write the version in the database file</summary>
		public virtual void WriteVersion(bool writeInTransaction)
		{
			fsi.SetWritePosition(NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.DatabaseHeaderVersionPosition, writeInTransaction);
			fsi.WriteInt(NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant.CurrentFileFormatVersion
				, writeInTransaction, "database file format version");
			storageEngine.SetVersion(NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.CurrentFileFormatVersion);
		}

		public virtual NeoDatis.Odb.DatabaseId WriteDatabaseId(long creationDate, bool writeInTransaction
			)
		{
			NeoDatis.Odb.DatabaseId databaseId = NeoDatis.Odb.Impl.Tool.UUID.GetDatabaseId(creationDate
				);
			fsi.WriteLong(databaseId.GetIds()[0], writeInTransaction, "database id 1/4", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
				.DirectWriteAction);
			fsi.WriteLong(databaseId.GetIds()[1], writeInTransaction, "database id 2/4", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
				.DirectWriteAction);
			fsi.WriteLong(databaseId.GetIds()[2], writeInTransaction, "database id 3/4", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
				.DirectWriteAction);
			fsi.WriteLong(databaseId.GetIds()[3], writeInTransaction, "database id 4/4", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
				.DirectWriteAction);
			storageEngine.SetDatabaseId(databaseId);
			return databaseId;
		}

		/// <summary>Write the replication flag : 0= No replication, 1= Use replication</summary>
		public virtual void WriteReplicationFlag(bool useReplication, bool writeInTransaction
			)
		{
			fsi.SetWritePosition(NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.DatabaseHeaderUseReplicationPosition, writeInTransaction);
			fsi.WriteByte(useReplication ? NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.WithReplication : NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.NoReplication, writeInTransaction, "replication flag");
		}

		/// <summary>Write the current transaction Id, out of transaction</summary>
		/// <param name="transactionId"></param>
		/// <></>
		public virtual void WriteLastTransactionId(NeoDatis.Odb.TransactionId transactionId
			)
		{
			fsi.SetWritePosition(NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.DatabaseHeaderLastTransactionId, false);
			// FIXME This should always be written directly without transaction
			fsi.WriteLong(transactionId.GetId1(), false, "last transaction id 1/2", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
				.DirectWriteAction);
			fsi.WriteLong(transactionId.GetId2(), false, "last transaction id 2/2", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
				.DirectWriteAction);
		}

		/// <summary>Write the number of classes in meta-model</summary>
		public virtual void WriteNumberOfClasses(long number, bool writeInTransaction)
		{
			fsi.SetWritePosition(NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.DatabaseHeaderNumberOfClassesPosition, writeInTransaction);
			fsi.WriteLong(number, writeInTransaction, "nb classes", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
				.DirectWriteAction);
		}

		/// <summary>Write the status of the last odb close</summary>
		public virtual void WriteLastODBCloseStatus(bool ok, bool writeInTransaction)
		{
			fsi.SetWritePosition(NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.DatabaseHeaderLastCloseStatusPosition, writeInTransaction);
			fsi.WriteBoolean(ok, writeInTransaction, "odb last close status");
		}

		/// <summary>Write the database characterEncoding</summary>
		public virtual void WriteDatabaseCharacterEncoding(bool writeInTransaction)
		{
			fsi.SetWritePosition(NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.DatabaseHeaderDatabaseCharacterEncodingPosition, writeInTransaction);
			if (NeoDatis.Odb.OdbConfiguration.HasEncoding())
			{
				fsi.WriteString(NeoDatis.Odb.OdbConfiguration.GetDatabaseCharacterEncoding(), writeInTransaction
					, true, 50);
			}
			else
			{
				fsi.WriteString(NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
					.NoEncoding, writeInTransaction, false, 50);
			}
		}

		/// <summary>
		/// Writes the header of a block of type ID - a block that contains ids of
		/// objects and classes
		/// </summary>
		/// <param name="position">
		/// Position at which the block must be written, if -1, take the
		/// next available position
		/// </param>
		/// <param name="idBlockSize">The block size in byte</param>
		/// <param name="blockStatus">The block status</param>
		/// <param name="blockNumber">The number of the block</param>
		/// <param name="previousBlockPosition">The position of the previous block of the same type
		/// 	</param>
		/// <param name="writeInTransaction">To indicate if write must be done in transaction
		/// 	</param>
		/// <returns>The position of the id @</returns>
		public virtual long WriteIdBlock(long position, int idBlockSize, byte blockStatus
			, int blockNumber, long previousBlockPosition, bool writeInTransaction)
		{
			if (position == -1)
			{
				position = fsi.GetAvailablePosition();
			}
			// LogUtil.fileSystemOn(true);
			// Updates the database header with the current id block position
			fsi.SetWritePosition(NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.DatabaseHeaderCurrentIdBlockPosition, writeInTransaction);
			fsi.WriteLong(position, false, "current id block position", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
				.DirectWriteAction);
			fsi.SetWritePosition(position, writeInTransaction);
			fsi.WriteInt(idBlockSize, writeInTransaction, "block size");
			// LogUtil.fileSystemOn(false);
			fsi.WriteByte(NeoDatis.Odb.Impl.Core.Layers.Layer3.Block.BlockTypes.BlockTypeIds, 
				writeInTransaction);
			fsi.WriteByte(blockStatus, writeInTransaction);
			// prev position
			fsi.WriteLong(previousBlockPosition, writeInTransaction, "prev block pos", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
				.DirectWriteAction);
			// next position
			fsi.WriteLong(-1, writeInTransaction, "next block pos", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
				.DirectWriteAction);
			fsi.WriteInt(blockNumber, writeInTransaction, "id block number");
			fsi.WriteLong(0, writeInTransaction, "id block max id", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
				.DirectWriteAction);
			fsi.SetWritePosition(position + NeoDatis.Odb.OdbConfiguration.GetIdBlockSize() - 
				1, writeInTransaction);
			fsi.WriteByte((byte)0, writeInTransaction);
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogIdDebug))
			{
				NeoDatis.Tool.DLogger.Debug(DepthToSpaces() + "After create block, available position is "
					 + fsi.GetAvailablePosition());
			}
			return position;
		}

		/// <summary>
		/// Marks a block of type id as full, changes the status and the next block
		/// position
		/// </summary>
		/// <param name="blockPosition"></param>
		/// <param name="nextBlockPosition"></param>
		/// <param name="writeInTransaction"></param>
		/// <returns>The block position @</returns>
		public virtual long MarkIdBlockAsFull(long blockPosition, long nextBlockPosition, 
			bool writeInTransaction)
		{
			fsi.SetWritePosition(blockPosition + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.BlockIdOffsetForBlockStatus, writeInTransaction);
			fsi.WriteByte(NeoDatis.Odb.Impl.Core.Layers.Layer3.Block.BlockStatus.BlockFull, writeInTransaction
				);
			fsi.SetWritePosition(blockPosition + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.BlockIdOffsetForNextBlock, writeInTransaction);
			fsi.WriteLong(nextBlockPosition, writeInTransaction, "next id block pos", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
				.DirectWriteAction);
			return blockPosition;
		}

		/// <summary>Associate an object OID to its position</summary>
		/// <param name="idType">The type : can be object or class</param>
		/// <param name="idStatus">The status of the OID</param>
		/// <param name="currentBlockIdPosition">The current OID block position</param>
		/// <param name="oid">The OID</param>
		/// <param name="objectPosition">The position</param>
		/// <param name="writeInTransaction">To indicate if write must be executed in transaction
		/// 	</param>
		/// <returns>@</returns>
		public virtual long AssociateIdToObject(byte idType, byte idStatus, long currentBlockIdPosition
			, NeoDatis.Odb.OID oid, long objectPosition, bool writeInTransaction)
		{
			// Update the max id of the current block
			fsi.SetWritePosition(currentBlockIdPosition + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.BlockIdOffsetForMaxId, writeInTransaction);
			fsi.WriteLong(oid.GetObjectId(), writeInTransaction, "id block max id update", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
				.PointerWriteAction);
			long l1 = (oid.GetObjectId() - 1) % NeoDatis.Odb.OdbConfiguration.GetNB_IDS_PER_BLOCK
				();
			long l2 = NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant.BlockIdOffsetForStartOfRepetition;
			long idPosition = currentBlockIdPosition + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.BlockIdOffsetForStartOfRepetition + (l1) * NeoDatis.Odb.OdbConfiguration.GetID_BLOCK_REPETITION_SIZE
				();
			// go to the next id position
			fsi.SetWritePosition(idPosition, writeInTransaction);
			// id type
			fsi.WriteByte(idType, writeInTransaction, "id type");
			// id
			fsi.WriteLong(oid.GetObjectId(), writeInTransaction, "oid", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
				.PointerWriteAction);
			// id status
			fsi.WriteByte(idStatus, writeInTransaction, "id status");
			// object position
			fsi.WriteLong(objectPosition, writeInTransaction, "obj pos", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
				.PointerWriteAction);
			return idPosition;
		}

		/// <summary>Updates the real object position of the object OID</summary>
		/// <param name="idPosition">The OID position</param>
		/// <param name="objectPosition">The real object position</param>
		/// <param name="writeInTransactionTo">indicate if write must be done in transaction @
		/// 	</param>
		public virtual void UpdateObjectPositionForObjectOIDWithPosition(long idPosition, 
			long objectPosition, bool writeInTransaction)
		{
			fsi.SetWritePosition(idPosition, writeInTransaction);
			fsi.WriteByte(NeoDatis.Odb.Core.Layers.Layer3.IDTypes.Object, writeInTransaction, 
				"id type");
			fsi.SetWritePosition(idPosition + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.BlockIdRepetitionIdStatus, writeInTransaction);
			fsi.WriteByte(NeoDatis.Odb.Impl.Core.Layers.Layer3.Oid.IDStatus.Active, writeInTransaction
				);
			fsi.WriteLong(objectPosition, writeInTransaction, "Updating object position of id"
				, NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction.PointerWriteAction);
		}

		/// <summary>Udates the real class positon of the class OID</summary>
		/// <param name="idPosition"></param>
		/// <param name="objectPosition"></param>
		/// <param name="writeInTransaction"></param>
		/// <></>
		public virtual void UpdateClassPositionForClassOIDWithPosition(long idPosition, long
			 objectPosition, bool writeInTransaction)
		{
			fsi.SetWritePosition(idPosition, writeInTransaction);
			fsi.WriteByte(NeoDatis.Odb.Core.Layers.Layer3.IDTypes.Class, writeInTransaction, 
				"id type");
			fsi.SetWritePosition(idPosition + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.BlockIdRepetitionIdStatus, writeInTransaction);
			fsi.WriteByte(NeoDatis.Odb.Impl.Core.Layers.Layer3.Oid.IDStatus.Active, writeInTransaction
				);
			fsi.WriteLong(objectPosition, writeInTransaction, "Updating class position of id"
				, NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction.PointerWriteAction);
		}

		public virtual void UpdateStatusForIdWithPosition(long idPosition, byte newStatus
			, bool writeInTransaction)
		{
			fsi.SetWritePosition(idPosition + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.BlockIdRepetitionIdStatus, writeInTransaction);
			fsi.WriteByte(newStatus, writeInTransaction, "Updating id status");
		}

		/// <summary>Persist a single class info - This method is used by the XML Importer.</summary>
		/// <remarks>Persist a single class info - This method is used by the XML Importer.</remarks>
		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo PersistClass(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			 newClassInfo, int lastClassInfoIndex, bool addClass, bool addDependentClasses)
		{
			NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel metaModel = GetSession().GetMetaModel
				();
			NeoDatis.Odb.OID classInfoId = newClassInfo.GetId();
			if (classInfoId == null)
			{
				classInfoId = GetIdManager().GetNextClassId(-1);
				newClassInfo.SetId(classInfoId);
			}
			long writePosition = fsi.GetAvailablePosition();
			newClassInfo.SetPosition(writePosition);
			GetIdManager().UpdateClassPositionForId(classInfoId, writePosition, true);
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
			{
				NeoDatis.Tool.DLogger.Debug("Persisting class into database : " + newClassInfo.GetFullClassName
					() + " with oid " + classInfoId + " at pos " + writePosition);
				NeoDatis.Tool.DLogger.Debug("class " + newClassInfo.GetFullClassName() + " has " 
					+ newClassInfo.GetNumberOfAttributes() + " attributes : " + newClassInfo.GetAttributes
					());
			}
			// The class info oid is created in ObjectWriter.writeClassInfoHeader
			if (metaModel.GetNumberOfClasses() > 0 && lastClassInfoIndex != -2)
			{
				NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo lastClassinfo = null;
				if (lastClassInfoIndex == -1)
				{
					lastClassinfo = metaModel.GetLastClassInfo();
				}
				else
				{
					lastClassinfo = metaModel.GetClassInfo(lastClassInfoIndex);
				}
				lastClassinfo.SetNextClassOID(newClassInfo.GetId());
				if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
				{
					NeoDatis.Tool.DLogger.Debug("changing next class oid. of class info " + lastClassinfo
						.GetFullClassName() + " @ " + lastClassinfo.GetPosition() + " + offset " + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
						.ClassOffsetNextClassPosition + " to " + newClassInfo.GetId() + "(" + newClassInfo
						.GetFullClassName() + ")");
				}
				fsi.SetWritePosition(lastClassinfo.GetPosition() + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
					.ClassOffsetNextClassPosition, true);
				fsi.WriteLong(newClassInfo.GetId().GetObjectId(), true, "next class oid", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
					.PointerWriteAction);
				newClassInfo.SetPreviousClassOID(lastClassinfo.GetId());
			}
			if (addClass)
			{
				metaModel.AddClass(newClassInfo);
			}
			// updates number of classes
			WriteNumberOfClasses(metaModel.GetNumberOfClasses(), true);
			// If it is the first class , updates the first class OID
			if (newClassInfo.GetPreviousClassOID() == null)
			{
				WriteFirstClassInfoOID(newClassInfo.GetId(), true);
			}
			// Writes the header of the class - out of transaction (FIXME why out of
			// transaction)
			WriteClassInfoHeader(newClassInfo, writePosition, false);
			if (addDependentClasses)
			{
				NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassAttributeInfo
					> dependingAttributes = newClassInfo.GetAllNonNativeAttributes();
				NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassAttributeInfo cai = null;
				for (int i = 0; i < dependingAttributes.Count; i++)
				{
					cai = dependingAttributes[i];
					try
					{
						NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo existingCI = metaModel.GetClassInfo
							(cai.GetFullClassname(), false);
						if (existingCI == null)
						{
							// TODO check if this getClassInfo is ok. Maybe, should
							// use
							// a buffered one
							AddClasses(classIntrospector.Introspect(cai.GetFullClassname(), true));
						}
						else
						{
							// Even,if it exist,take the one from metamodel
							cai.SetClassInfo(existingCI);
						}
					}
					catch (System.Exception e)
					{
						throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ClassIntrospectionError
							.AddParameter(cai.GetFullClassname()), e);
					}
				}
			}
			WriteClassInfoBody(newClassInfo, fsi.GetAvailablePosition(), true);
			return newClassInfo;
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo AddClass(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			 newClassInfo, bool addDependentClasses)
		{
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo classInfo = GetSession().GetMetaModel
				().GetClassInfo(newClassInfo.GetFullClassName(), false);
			if (classInfo != null && classInfo.GetPosition() != -1)
			{
				return classInfo;
			}
			return PersistClass(newClassInfo, -1, true, addDependentClasses);
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoList AddClasses(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoList
			 classInfoList)
		{
			System.Collections.IEnumerator iterator = classInfoList.GetClassInfos().GetEnumerator
				();
			while (iterator.MoveNext())
			{
				AddClass((NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo)iterator.Current, true);
			}
			return classInfoList;
		}

		public virtual void WriteClassInfoHeader(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			 classInfo, long position, bool writeInTransaction)
		{
			NeoDatis.Odb.OID classId = classInfo.GetId();
			if (classId == null)
			{
				classId = idManager.GetNextClassId(position);
				classInfo.SetId(classId);
			}
			else
			{
				idManager.UpdateClassPositionForId(classId, position, true);
			}
			fsi.SetWritePosition(position, writeInTransaction);
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
			{
				NeoDatis.Tool.DLogger.Debug(DepthToSpaces() + "Writing new Class info header at "
					 + position + " : " + classInfo.ToString());
			}
			// Real value of block size is only known at the end of the writing
			fsi.WriteInt(0, writeInTransaction, "block size");
			fsi.WriteByte(NeoDatis.Odb.Impl.Core.Layers.Layer3.Block.BlockTypes.BlockTypeClassHeader
				, writeInTransaction, "class header block type");
			fsi.WriteByte(classInfo.GetClassCategory(), writeInTransaction, "Class info category"
				);
			fsi.WriteLong(classId.GetObjectId(), writeInTransaction, "class id", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
				.DataWriteAction);
			WriteOid(classInfo.GetPreviousClassOID(), writeInTransaction, "prev class oid", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
				.DataWriteAction);
			WriteOid(classInfo.GetNextClassOID(), writeInTransaction, "next class oid", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
				.DataWriteAction);
			fsi.WriteLong(classInfo.GetCommitedZoneInfo().GetNbObjects(), writeInTransaction, 
				"class nb objects", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction.DataWriteAction
				);
			WriteOid(classInfo.GetCommitedZoneInfo().first, writeInTransaction, "class first obj pos"
				, NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction.DataWriteAction);
			WriteOid(classInfo.GetCommitedZoneInfo().last, writeInTransaction, "class last obj pos"
				, NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction.DataWriteAction);
			// FIXME : append extra info if not empty (.net compatibility)
			fsi.WriteString(classInfo.GetFullClassName(), false, writeInTransaction);
			fsi.WriteInt(classInfo.GetMaxAttributeId(), writeInTransaction, "Max attribute id"
				);
			if (classInfo.GetAttributesDefinitionPosition() != -1)
			{
				fsi.WriteLong(classInfo.GetAttributesDefinitionPosition(), writeInTransaction, "class att def pos"
					, NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction.DataWriteAction);
			}
			else
			{
				// @todo check this
				fsi.WriteLong(-1, writeInTransaction, "class att def pos", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
					.DataWriteAction);
			}
			int blockSize = (int)(fsi.GetPosition() - position);
			WriteBlockSizeAt(position, blockSize, writeInTransaction, classInfo);
		}

		public virtual void EncodeOid(NeoDatis.Odb.OID oid, byte[] bytes, int offset)
		{
			if (oid == null)
			{
				byteArrayConverter.LongToByteArray(-1, bytes, offset);
			}
			else
			{
				// fsi.writeLong(-1, writeInTransaction, label, writeAction);
				byteArrayConverter.LongToByteArray(oid.GetObjectId(), bytes, offset);
			}
		}

		// fsi.writeLong(oid.getObjectId(), writeInTransaction, label,
		// writeAction);
		public virtual void WriteOid(NeoDatis.Odb.OID oid, bool writeInTransaction, string
			 label, int writeAction)
		{
			if (oid == null)
			{
				fsi.WriteLong(-1, writeInTransaction, label, writeAction);
			}
			else
			{
				fsi.WriteLong(oid.GetObjectId(), writeInTransaction, label, writeAction);
			}
		}

		/// <summary>Write the class info body to the database file.</summary>
		/// <remarks>
		/// Write the class info body to the database file. TODO Check if we really
		/// must recall the writeClassInfoHeader
		/// </remarks>
		/// <param name="classInfo"></param>
		/// <param name="position">The position</param>
		/// <param name="writeInTransaction"></param>
		/// <></>
		public virtual void WriteClassInfoBody(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			 classInfo, long position, bool writeInTransaction)
		{
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
			{
				NeoDatis.Tool.DLogger.Debug(DepthToSpaces() + "Writing new Class info body at " +
					 position + " : " + classInfo.ToString());
			}
			// updates class info
			classInfo.SetAttributesDefinitionPosition(position);
			// FIXME : change this to write only the position and not the whole
			// header
			WriteClassInfoHeader(classInfo, classInfo.GetPosition(), writeInTransaction);
			fsi.SetWritePosition(position, writeInTransaction);
			// block definition
			fsi.WriteInt(0, writeInTransaction, "block size");
			fsi.WriteByte(NeoDatis.Odb.Impl.Core.Layers.Layer3.Block.BlockTypes.BlockTypeClassBody
				, writeInTransaction);
			// number of attributes
			fsi.WriteLong(classInfo.GetAttributes().Count, writeInTransaction, "class nb attributes"
				, NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction.DataWriteAction);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassAttributeInfo cai = null;
			for (int i = 0; i < classInfo.GetAttributes().Count; i++)
			{
				cai = classInfo.GetAttributes()[i];
				WriteClassAttributeInfo(cai, writeInTransaction);
			}
			int blockSize = (int)(fsi.GetPosition() - position);
			WriteBlockSizeAt(position, blockSize, writeInTransaction, classInfo);
		}

		public virtual long WriteClassInfoIndexes(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			 classInfo)
		{
			bool writeInTransaction = true;
			long position = fsi.GetAvailablePosition();
			fsi.SetWritePosition(position, writeInTransaction);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoIndex cii = null;
			long previousIndexPosition = -1;
			long currentIndexPosition = position;
			long nextIndexPosition = -1;
			long currentPosition = -1;
			for (int i = 0; i < classInfo.GetNumberOfIndexes(); i++)
			{
				currentIndexPosition = fsi.GetPosition();
				cii = classInfo.GetIndex(i);
				fsi.WriteInt(0, writeInTransaction, "block size");
				fsi.WriteByte(NeoDatis.Odb.Impl.Core.Layers.Layer3.Block.BlockTypes.BlockTypeIndex
					, true, "Index block type");
				fsi.WriteLong(previousIndexPosition, writeInTransaction, "prev index pos", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
					.PointerWriteAction);
				// The next position is only know at the end of the write
				fsi.WriteLong(-1, writeInTransaction, "next index pos", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
					.PointerWriteAction);
				fsi.WriteString(cii.GetName(), false, writeInTransaction);
				fsi.WriteBoolean(cii.IsUnique(), writeInTransaction, "index is unique");
				fsi.WriteByte(cii.GetStatus(), writeInTransaction, "index status");
				fsi.WriteLong(cii.GetCreationDate(), writeInTransaction, "creation date", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
					.DataWriteAction);
				fsi.WriteLong(cii.GetLastRebuild(), writeInTransaction, "last rebuild", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
					.DataWriteAction);
				fsi.WriteInt(cii.GetAttributeIds().Length, writeInTransaction, "number of fields"
					);
				for (int j = 0; j < cii.GetAttributeIds().Length; j++)
				{
					fsi.WriteInt(cii.GetAttributeIds()[j], writeInTransaction, "attr id");
				}
				currentPosition = fsi.GetPosition();
				// Write the block size
				int blockSize = (int)(fsi.GetPosition() - currentIndexPosition);
				WriteBlockSizeAt(currentIndexPosition, blockSize, writeInTransaction, classInfo);
				// Write the next index position
				if (i + 1 < classInfo.GetNumberOfIndexes())
				{
					nextIndexPosition = currentPosition;
				}
				else
				{
					nextIndexPosition = -1;
				}
				// reset cursor to write the next position
				fsi.SetWritePosition(currentIndexPosition + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
					.Integer.GetSize() + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Byte.GetSize()
					 + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Long.GetSize(), writeInTransaction
					);
				fsi.WriteLong(nextIndexPosition, writeInTransaction, "next index pos", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
					.PointerWriteAction);
				previousIndexPosition = currentIndexPosition;
				// reset the write cursor
				fsi.SetWritePosition(currentPosition, writeInTransaction);
			}
			return position;
		}

		public virtual void UpdateClassInfo(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			 classInfo, bool writeInTransaction)
		{
			// first check dependent classes
			NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassAttributeInfo
				> dependingAttributes = classInfo.GetAllNonNativeAttributes();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel metaModel = GetSession().GetMetaModel
				();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassAttributeInfo cai = null;
			for (int i = 0; i < dependingAttributes.Count; i++)
			{
				cai = dependingAttributes[i];
				try
				{
					NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo existingCI = metaModel.GetClassInfo
						(cai.GetFullClassname(), false);
					if (existingCI == null)
					{
						// TODO check if this getClassInfo is ok. Maybe, should
						// use
						// a buffered one
						AddClasses(classIntrospector.Introspect(cai.GetFullClassname(), true));
					}
					else
					{
						// FIXME should we update class info?
						cai.SetClassInfo(existingCI);
					}
				}
				catch (System.Exception e)
				{
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ClassIntrospectionError
						.AddParameter(cai.GetFullClassname()), e);
				}
			}
			// To force the rewrite of class info body
			classInfo.SetAttributesDefinitionPosition(-1);
			long newCiPosition = fsi.GetAvailablePosition();
			classInfo.SetPosition(newCiPosition);
			WriteClassInfoHeader(classInfo, newCiPosition, writeInTransaction);
			WriteClassInfoBody(classInfo, fsi.GetAvailablePosition(), writeInTransaction);
		}

		/// <summary>Resets the position of the first class of the metamodel.</summary>
		/// <remarks>
		/// Resets the position of the first class of the metamodel. It Happens when
		/// database is being refactored
		/// </remarks>
		/// <param name="classInfoPosition"></param>
		/// <></>
		public virtual void WriteFirstClassInfoOID(NeoDatis.Odb.OID classInfoID, bool inTransaction
			)
		{
			long positionToWrite = NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.DatabaseHeaderFirstClassOid;
			fsi.SetWritePosition(positionToWrite, inTransaction);
			WriteOid(classInfoID, inTransaction, "first class info oid", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
				.DataWriteAction);
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
			{
				NeoDatis.Tool.DLogger.Debug(DepthToSpaces() + "Updating first class info oid at "
					 + positionToWrite + " with oid " + classInfoID);
			}
		}

		private void UpdateNextClassInfoPositionOfClassInfo(long classInfoPosition, long 
			newCiPosition)
		{
			fsi.SetWritePosition(classInfoPosition + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.ClassOffsetNextClassPosition, true);
			fsi.WriteLong(newCiPosition, true, "new next ci position", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
				.DataWriteAction);
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
			{
				NeoDatis.Tool.DLogger.Debug(DepthToSpaces() + "Updating next class info of class info at "
					 + classInfoPosition + " with " + classInfoPosition);
			}
		}

		private void UpdatePreviousClassInfoPositionOfClassInfo(long classInfoPosition, long
			 newCiPosition)
		{
			fsi.SetWritePosition(classInfoPosition + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.ClassOffsetPreviousClassPosition, true);
			fsi.WriteLong(newCiPosition, true, "new prev ci position", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
				.DataWriteAction);
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
			{
				NeoDatis.Tool.DLogger.Debug(DepthToSpaces() + "Updating prev class info of class info at "
					 + classInfoPosition + " with " + classInfoPosition);
			}
		}

		/// <summary>Writes a class attribute info, an attribute of a class</summary>
		/// <param name="cai"></param>
		/// <param name="writeInTransaction"></param>
		/// <></>
		private void WriteClassAttributeInfo(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassAttributeInfo
			 cai, bool writeInTransaction)
		{
			fsi.WriteInt(cai.GetId(), writeInTransaction, "attribute id");
			fsi.WriteBoolean(cai.IsNative(), writeInTransaction);
			if (cai.IsNative())
			{
				fsi.WriteInt(cai.GetAttributeType().GetId(), writeInTransaction, "att odb type id"
					);
				if (cai.GetAttributeType().IsArray())
				{
					fsi.WriteInt(cai.GetAttributeType().GetSubType().GetId(), writeInTransaction, "att array sub type"
						);
					// when the attribute is not native, then write its class info
					// position
					if (cai.GetAttributeType().GetSubType().IsNonNative())
					{
						fsi.WriteLong(storageEngine.GetSession(true).GetMetaModel().GetClassInfo(cai.GetAttributeType
							().GetSubType().GetName(), true).GetId().GetObjectId(), writeInTransaction, "class info id of array subtype"
							, NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction.DataWriteAction);
					}
				}
				// For enum, we write the class info id of the enum class
				if (cai.GetAttributeType().IsEnum())
				{
					fsi.WriteLong(storageEngine.GetSession(true).GetMetaModel().GetClassInfo(cai.GetFullClassname
						(), true).GetId().GetObjectId(), writeInTransaction, "class info id", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
						.DataWriteAction);
				}
			}
			else
			{
				fsi.WriteLong(storageEngine.GetSession(true).GetMetaModel().GetClassInfo(cai.GetFullClassname
					(), true).GetId().GetObjectId(), writeInTransaction, "class info id", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
					.DataWriteAction);
			}
			fsi.WriteString(cai.GetName(), false, writeInTransaction);
			fsi.WriteBoolean(cai.IsIndex(), writeInTransaction);
		}

		/// <summary>Actually write the object data to the database file</summary>
		/// <param name="oidOfObjectToQuery">The object id, can be -1 (not set)</param>
		/// <param name="aoi">The object meta infor The object info to be written</param>
		/// <param name="position">if -1, it is a new instance, if not, it is an update</param>
		/// <param name="updatePointers"></param>
		/// <returns>The object posiiton or id(if &lt;0)</returns>
		/// <exception cref="System.Exception">System.Exception</exception>
		/// <>
		/// * public OID writeObjectInfo(OID oid, AbstractObjectInfo
		/// aoi, long position, boolean updatePointers) throws Exception
		/// { currentDepth++;
		/// try {
		/// if (aoi.isNative()) { return
		/// writeNativeObjectInfo((NativeObjectInfo) aoi, position,
		/// updatePointers, false); }
		/// return writeNonNativeObjectInfo(oid, aoi, position,
		/// updatePointers, false); } finally { currentDepth--; } }
		/// </>
		private long WriteNativeObjectInfo(NeoDatis.Odb.Core.Layers.Layer2.Meta.NativeObjectInfo
			 noi, long position, bool updatePointers, bool writeInTransaction)
		{
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogIdDebug))
			{
				NeoDatis.Tool.DLogger.Debug(DepthToSpaces() + "Writing native object at " + position
					 + " : Type=" + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.GetNameFromId(noi.GetOdbTypeId
					()) + " | Value=" + noi.ToString());
			}
			if (noi.IsAtomicNativeObject())
			{
				return WriteAtomicNativeObject((NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo
					)noi, writeInTransaction);
			}
			if (noi.IsNull())
			{
				WriteNullNativeObjectHeader(noi.GetOdbTypeId(), writeInTransaction);
				return position;
			}
			if (noi.IsCollectionObject())
			{
				return WriteCollection((NeoDatis.Odb.Core.Layers.Layer2.Meta.CollectionObjectInfo
					)noi, writeInTransaction);
			}
			if (noi.IsMapObject())
			{
				return WriteMap((NeoDatis.Odb.Core.Layers.Layer2.Meta.MapObjectInfo)noi, writeInTransaction
					);
			}
			if (noi.IsArrayObject())
			{
				return WriteArray((NeoDatis.Odb.Core.Layers.Layer2.Meta.ArrayObjectInfo)noi, writeInTransaction
					);
			}
			if (noi.IsEnumObject())
			{
				return WriteEnumNativeObject((NeoDatis.Odb.Core.Layers.Layer2.Meta.EnumNativeObjectInfo
					)noi, writeInTransaction);
			}
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.NativeTypeNotSupported
				.AddParameter(noi.GetOdbTypeId()));
		}

		public virtual NeoDatis.Odb.OID WriteNonNativeObjectInfo(NeoDatis.Odb.OID existingOid
			, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo objectInfo, long position
			, bool writeDataInTransaction, bool isNewObject)
		{
			NeoDatis.Odb.Core.Transaction.ISession lsession = GetSession();
			NeoDatis.Odb.Core.Transaction.ICache cache = lsession.GetCache();
			bool hasObject = objectInfo.GetObject() != null;
			// Insert triggers for CS Mode, local mode insert triggers are called in the DefaultInstrumentationCallbackForStore class
			if (isNewObject && !isLocalMode)
			{
				triggerManager.ManageInsertTriggerBefore(objectInfo.GetClassInfo().GetFullClassName
					(), objectInfo);
			}
			// Checks if object is null,for null objects,there is nothing to do
			if (objectInfo.IsNull())
			{
				return NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant.NullObjectId;
			}
			NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel metaModel = lsession.GetMetaModel(
				);
			// first checks if the class of this object already exist in the
			// metamodel
			if (!metaModel.ExistClass(objectInfo.GetClassInfo().GetFullClassName()))
			{
				AddClass(objectInfo.GetClassInfo(), true);
			}
			// if position is -1, gets the position where to write the object
			if (position == -1)
			{
				// Write at the end of the file
				position = fsi.GetAvailablePosition();
				// Updates the meta object position
				objectInfo.SetPosition(position);
			}
			// Gets the object id
			NeoDatis.Odb.OID oid = existingOid;
			if (oid == null)
			{
				// If, to get the next id, a new id block must be created, then
				// there is an extra work
				// to update the current object position
				if (idManager.MustShift())
				{
					oid = idManager.GetNextObjectId(position);
					// The id manager wrote in the file so the position for the
					// object must be re-computed
					position = fsi.GetAvailablePosition();
					// The oid must be associated to this new position - id
					// operations are always out of transaction
					// in this case, the update is done out of the transaction as a
					// rollback won t need to
					// undo this. We are just creating the id
					// => third parameter(write in transaction) = false
					idManager.UpdateObjectPositionForOid(oid, position, false);
				}
				else
				{
					oid = idManager.GetNextObjectId(position);
				}
			}
			else
			{
				// If an oid was passed, it is because object already exist and
				// is being updated. So we
				// must update the object position
				// Here the update of the position of the id must be done in
				// transaction as the object
				// position of the id is being updated, and a rollback should undo
				// this
				// => third parameter(write in transaction) = true
				idManager.UpdateObjectPositionForOid(oid, position, true);
				// Keep the relation of id and position in the cache until the
				// commit
				cache.SavePositionOfObjectWithOid(oid, position);
			}
			// Sets the oid of the object in the inserting cache
			cache.UpdateIdOfInsertingObject(objectInfo.GetObject(), oid);
			// Only add the oid to unconnected zone if it is a new object
			if (isNewObject)
			{
				cache.AddOIDToUnconnectedZone(oid);
				if (NeoDatis.Odb.OdbConfiguration.ReconnectObjectsToSession())
				{
					NeoDatis.Odb.Core.Transaction.ICrossSessionCache crossSessionCache = NeoDatis.Odb.Impl.Core.Transaction.CacheFactory
						.GetCrossSessionCache(storageEngine.GetBaseIdentification().GetIdentification());
					crossSessionCache.AddObject(objectInfo.GetObject(), oid);
				}
			}
			objectInfo.SetOid(oid);
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
			{
				NeoDatis.Tool.DLogger.Debug(DepthToSpaces() + "Start Writing non native object of type "
					 + objectInfo.GetClassInfo().GetFullClassName() + " at " + position + " , oid = "
					 + oid + " : " + objectInfo.ToString());
			}
			if (objectInfo.GetClassInfo() == null || objectInfo.GetClassInfo().GetId() == null)
			{
				if (objectInfo.GetClassInfo() != null)
				{
					NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo clinfo = storageEngine.GetSession(
						true).GetMetaModel().GetClassInfo(objectInfo.GetClassInfo().GetFullClassName(), 
						true);
					objectInfo.SetClassInfo(clinfo);
				}
				else
				{
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.UndefinedClassInfo
						.AddParameter(objectInfo.ToString()));
				}
			}
			// updates the meta model - If class already exist, it returns the
			// metamodel class, which contains
			// a bit more informations
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo classInfo = AddClass(objectInfo.GetClassInfo
				(), true);
			objectInfo.SetClassInfo(classInfo);
			// 
			if (isNewObject)
			{
				ManageNewObjectPointers(objectInfo, classInfo, position, metaModel);
			}
			if (NeoDatis.Odb.OdbConfiguration.SaveHistory())
			{
				classInfo.AddHistory(new NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.History.InsertHistoryInfo
					("insert", oid, position, objectInfo.GetPreviousObjectOID(), objectInfo.GetNextObjectOID
					()));
			}
			fsi.SetWritePosition(position, writeDataInTransaction);
			objectInfo.SetPosition(position);
			int nbAttributes = objectInfo.GetClassInfo().GetAttributes().Count;
			// compute the size of the array of byte needed till the attibute
			// positions
			// BlockSize + Block Type + ObjectId + ClassInfoId + Previous + Next +
			// CreatDate + UpdateDate + VersionNumber + ObjectRef + isSync + NbAttri
			// + Attributes
			// Int + Int + Long + Long + Long + Long + Long + Long + int + Long +
			// Bool + int + variable
			// 7 Longs + 4Ints + 1Bool + variable
			int tsize = 7 * NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.SizeOfLong + 3 * NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
				.SizeOfInt + 2 * NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.SizeOfByte;
			byte[] bytes = new byte[tsize];
			// Block size
			byteArrayConverter.IntToByteArray(0, bytes, 0);
			// Block type
			bytes[4] = NeoDatis.Odb.Impl.Core.Layers.Layer3.Block.BlockTypes.BlockTypeNonNativeObject;
			// fsi.writeInt(BlockTypes.BLOCK_TYPE_NON_NATIVE_OBJECT,
			// writeDataInTransaction, "block size");
			// The object id
			EncodeOid(oid, bytes, 5);
			// fsi.writeLong(oid.getObjectId(), writeDataInTransaction, "oid",
			// DefaultWriteAction.DATA_WRITE_ACTION);
			// Class info id
			byteArrayConverter.LongToByteArray(classInfo.GetId().GetObjectId(), bytes, 13);
			// fsi.writeLong(classInfo.getId().getObjectId(),
			// writeDataInTransaction, "class info id",
			// DefaultWriteAction.DATA_WRITE_ACTION);
			// previous instance
			EncodeOid(objectInfo.GetPreviousObjectOID(), bytes, 21);
			// writeOid(objectInfo.getPreviousObjectOID(), writeDataInTransaction,
			// "prev instance", DefaultWriteAction.DATA_WRITE_ACTION);
			// next instance
			EncodeOid(objectInfo.GetNextObjectOID(), bytes, 29);
			// writeOid(objectInfo.getNextObjectOID(), writeDataInTransaction,
			// "next instance", DefaultWriteAction.DATA_WRITE_ACTION);
			// creation date, for update operation must be the original one
			byteArrayConverter.LongToByteArray(objectInfo.GetHeader().GetCreationDate(), bytes
				, 37);
			// fsi.writeLong(objectInfo.getHeader().getCreationDate(),
			// writeDataInTransaction, "creation date",
			// DefaultWriteAction.DATA_WRITE_ACTION);
			byteArrayConverter.LongToByteArray(NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs
				(), bytes, 45);
			// fsi.writeLong(OdbTime.getCurrentTimeInMs(), writeDataInTransaction,
			// "update date", DefaultWriteAction.DATA_WRITE_ACTION);
			// TODO check next version number
			byteArrayConverter.IntToByteArray(objectInfo.GetHeader().GetObjectVersion(), bytes
				, 53);
			// fsi.writeInt(objectInfo.getHeader().getObjectVersion(),
			// writeDataInTransaction, "object version number");
			// not used yet. But it will point to an internal object of type
			// ObjectReference that will have details on the references:
			// All the objects that point to it: to enable object integrity
			byteArrayConverter.LongToByteArray(-1, bytes, 57);
			// fsi.writeLong(-1, writeDataInTransaction, "object reference pointer",
			// DefaultWriteAction.DATA_WRITE_ACTION);
			// True if this object have been synchronized with main database, else
			// false
			byteArrayConverter.BooleanToByteArray(false, bytes, 65);
			// fsi.writeBoolean(false, writeDataInTransaction,
			// "is syncronized with external db");
			// now write the number of attributes and the position of all
			// attributes, we do not know them yet, so write 00 but at the end
			// of the write operation
			// These positions will be updated
			// The positions that is going to be written are 'int' representing
			// the offset position of the attribute
			// first write the number of attributes
			// fsi.writeInt(nbAttributes, writeDataInTransaction, "nb attr");
			byteArrayConverter.IntToByteArray(nbAttributes, bytes, 66);
			// Then write the array of bytes
			fsi.WriteBytes(bytes, writeDataInTransaction, "NonNativeObjectInfoHeader");
			// Store the position
			long attributePositionStart = fsi.GetPosition();
			int attributeSize = NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.SizeOfInt + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
				.SizeOfLong;
			byte[] abytes = new byte[nbAttributes * (attributeSize)];
			// here, just write an empty (0) array, as real values will be set at
			// the end
			fsi.WriteBytes(abytes, writeDataInTransaction, "Empty Attributes");
			long[] attributesIdentification = new long[nbAttributes];
			int[] attributeIds = new int[nbAttributes];
			// Puts the object info in the cache
			// storageEngine.getSession().getCache().addObject(position,
			// aoi.getObject(), objectInfo.getHeader());
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassAttributeInfo cai = null;
			NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo aoi2 = null;
			long nativeAttributePosition = -1;
			NeoDatis.Odb.OID nonNativeAttributeOid = null;
			long maxWritePosition = fsi.GetPosition();
			// Loop on all attributes
			for (int i = 0; i < nbAttributes; i++)
			{
				// Gets the attribute meta description
				cai = classInfo.GetAttributeInfo(i);
				// Gets the id of the attribute
				attributeIds[i] = cai.GetId();
				// Gets the attribute data
				aoi2 = objectInfo.GetAttributeValueFromId(cai.GetId());
				if (aoi2 == null)
				{
					// This only happens in 1 case : when a class has a field with
					// the same name of one of is superclass. In this, the deeper
					// attribute is null
					if (cai.IsNative())
					{
						aoi2 = new NeoDatis.Odb.Core.Layers.Layer2.Meta.NullNativeObjectInfo(cai.GetAttributeType
							().GetId());
					}
					else
					{
						aoi2 = new NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeNullObjectInfo(cai.GetClassInfo
							());
					}
				}
				if (aoi2.IsNative())
				{
					nativeAttributePosition = InternalStoreObject((NeoDatis.Odb.Core.Layers.Layer2.Meta.NativeObjectInfo
						)aoi2);
					// For native objects , odb stores their position
					attributesIdentification[i] = nativeAttributePosition;
				}
				else
				{
					if (aoi2.IsObjectReference())
					{
						NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectReference or = (NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectReference
							)aoi2;
						nonNativeAttributeOid = or.GetOid();
					}
					else
					{
						nonNativeAttributeOid = StoreObject(null, (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
							)aoi2);
					}
					// For non native objects , odb stores its oid as a negative
					// number!!u
					if (nonNativeAttributeOid != null)
					{
						attributesIdentification[i] = -nonNativeAttributeOid.GetObjectId();
					}
					else
					{
						attributesIdentification[i] = NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
							.NullObjectIdId;
					}
				}
				long p = fsi.GetPosition();
				if (p > maxWritePosition)
				{
					maxWritePosition = p;
				}
			}
			// Updates attributes identification in the object info header
			objectInfo.GetHeader().SetAttributesIdentification(attributesIdentification);
			objectInfo.GetHeader().SetAttributesIds(attributeIds);
			long positionAfterWrite = maxWritePosition;
			// Now writes back the attribute positions
			fsi.SetWritePosition(attributePositionStart, writeDataInTransaction);
			abytes = new byte[attributesIdentification.Length * (attributeSize)];
			for (int i = 0; i < attributesIdentification.Length; i++)
			{
				byteArrayConverter.IntToByteArray(attributeIds[i], abytes, i * attributeSize);
				byteArrayConverter.LongToByteArray(attributesIdentification[i], abytes, i * (attributeSize
					) + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.SizeOfInt);
				// fsi.writeInt(attributeIds[i], writeDataInTransaction, "attr id");
				// fsi.writeLong(attributesIdentification[i],
				// writeDataInTransaction, "att real pos",
				// DefaultWriteAction.DATA_WRITE_ACTION);
				// if (classInfo.getAttributeInfo(i).isNonNative() &&
				// attributesIdentification[i] > 0) {
				if (objectInfo.GetAttributeValueFromId(attributeIds[i]).IsNonNativeObject() && attributesIdentification
					[i] > 0)
				{
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.NonNativeAttributeStoredByPositionInsteadOfOid
						.AddParameter(classInfo.GetAttributeInfo(i).GetName()).AddParameter(classInfo.GetFullClassName
						()).AddParameter(attributesIdentification[i]));
				}
			}
			fsi.WriteBytes(abytes, writeDataInTransaction, "Filled Attributes");
			fsi.SetWritePosition(positionAfterWrite, writeDataInTransaction);
			int blockSize = (int)(positionAfterWrite - position);
			try
			{
				WriteBlockSizeAt(position, blockSize, writeDataInTransaction, objectInfo);
			}
			catch (NeoDatis.Odb.ODBRuntimeException e)
			{
				NeoDatis.Tool.DLogger.Debug("Error while writing block size. pos after write " + 
					positionAfterWrite + " / start pos = " + position);
				// throw new ODBRuntimeException(storageEngine,"Error while writing
				// block size. pos after write " + positionAfterWrite + " / start
				// pos = " + position,e);
				throw;
			}
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
			{
				NeoDatis.Tool.DLogger.Debug(DepthToSpaces() + "  Attributes positions of object with oid "
					 + oid + " are " + NeoDatis.Tool.DisplayUtility.LongArrayToString(attributesIdentification
					));
				NeoDatis.Tool.DLogger.Debug(DepthToSpaces() + "End Writing non native object at "
					 + position + " with oid " + oid + " - prev oid=" + objectInfo.GetPreviousObjectOID
					() + " / next oid=" + objectInfo.GetNextObjectOID());
				if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogIdDebug))
				{
					NeoDatis.Tool.DLogger.Debug(" - current buffer : " + fsi.GetIo().ToString());
				}
			}
			// Only insert in index for new objects
			if (isNewObject)
			{
				// insert object id in indexes, if exist
				ManageIndexesForInsert(oid, objectInfo);
				if (hasObject)
				{
					triggerManager.ManageInsertTriggerAfter(objectInfo.GetClassInfo().GetFullClassName
						(), objectInfo.GetObject(), oid);
				}
				else
				{
					// triggers
					triggerManager.ManageInsertTriggerAfter(objectInfo.GetClassInfo().GetFullClassName
						(), objectInfo, oid);
				}
			}
			return oid;
		}

		public virtual NeoDatis.Odb.OID WriteNonNativeObjectInfoOld(NeoDatis.Odb.OID existingOid
			, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo objectInfo, long position
			, bool writeDataInTransaction, bool isNewObject)
		{
			NeoDatis.Odb.Core.Transaction.ISession lsession = GetSession();
			NeoDatis.Odb.Core.Transaction.ICache cache = lsession.GetCache();
			bool hasObject = objectInfo.GetObject() != null;
			if (isNewObject)
			{
				if (hasObject)
				{
					// triggers
					triggerManager.ManageInsertTriggerBefore(objectInfo.GetClassInfo().GetFullClassName
						(), objectInfo.GetObject());
				}
				else
				{
					triggerManager.ManageInsertTriggerBefore(objectInfo.GetClassInfo().GetFullClassName
						(), objectInfo);
				}
			}
			// Checks if object is null,for null objects,there is nothing to do
			if (objectInfo.IsNull())
			{
				return NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant.NullObjectId;
			}
			NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel metaModel = lsession.GetMetaModel(
				);
			// first checks if the class of this object already exist in the
			// metamodel
			if (!metaModel.ExistClass(objectInfo.GetClassInfo().GetFullClassName()))
			{
				AddClass(objectInfo.GetClassInfo(), true);
			}
			// if position is -1, gets the position where to write the object
			if (position == -1)
			{
				// Write at the end of the file
				position = fsi.GetAvailablePosition();
				// Updates the meta object position
				objectInfo.SetPosition(position);
			}
			// Gets the object id
			NeoDatis.Odb.OID oid = existingOid;
			if (oid == null)
			{
				// If, to get the next id, a new id block must be created, then
				// there is an extra work
				// to update the current object position
				if (idManager.MustShift())
				{
					oid = idManager.GetNextObjectId(position);
					// The id manager wrote in the file so the position for the
					// object must be re-computed
					position = fsi.GetAvailablePosition();
					// The oid must be associated to this new position - id
					// operations are always out of transaction
					// in this case, the update is done out of the transaction as a
					// rollback won t need to
					// undo this. We are just creating the id
					// => third parameter(write in transaction) = false
					idManager.UpdateObjectPositionForOid(oid, position, false);
				}
				else
				{
					oid = idManager.GetNextObjectId(position);
				}
			}
			else
			{
				// If an oid was passed, it is because object already exist and
				// is being updated. So we
				// must update the object position
				// Here the update of the position of the id must be done in
				// transaction as the object
				// position of the id is being updated, and a rollback should undo
				// this
				// => third parameter(write in transaction) = true
				idManager.UpdateObjectPositionForOid(oid, position, true);
				// Keep the relation of id and position in the cache until the
				// commit
				cache.SavePositionOfObjectWithOid(oid, position);
			}
			// Sets the oid of the object in the inserting cache
			cache.UpdateIdOfInsertingObject(objectInfo.GetObject(), oid);
			// Only add the oid to unconnected zone if it is a new object
			if (isNewObject)
			{
				cache.AddOIDToUnconnectedZone(oid);
				if (NeoDatis.Odb.OdbConfiguration.ReconnectObjectsToSession())
				{
					NeoDatis.Odb.Core.Transaction.ICrossSessionCache crossSessionCache = NeoDatis.Odb.Impl.Core.Transaction.CacheFactory
						.GetCrossSessionCache(storageEngine.GetBaseIdentification().GetIdentification());
					crossSessionCache.AddObject(objectInfo.GetObject(), oid);
				}
			}
			objectInfo.SetOid(oid);
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
			{
				NeoDatis.Tool.DLogger.Debug(DepthToSpaces() + "Start Writing non native object of type "
					 + objectInfo.GetClassInfo().GetFullClassName() + " at " + position + " , oid = "
					 + oid + " : " + objectInfo.ToString());
			}
			if (objectInfo.GetClassInfo() == null || objectInfo.GetClassInfo().GetId() == null)
			{
				if (objectInfo.GetClassInfo() != null)
				{
					NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo clinfo = storageEngine.GetSession(
						true).GetMetaModel().GetClassInfo(objectInfo.GetClassInfo().GetFullClassName(), 
						true);
					objectInfo.SetClassInfo(clinfo);
				}
				else
				{
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.UndefinedClassInfo
						.AddParameter(objectInfo.ToString()));
				}
			}
			// updates the meta model - If class already exist, it returns the
			// metamodel class, which contains
			// a bit more informations
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo classInfo = AddClass(objectInfo.GetClassInfo
				(), true);
			objectInfo.SetClassInfo(classInfo);
			// 
			if (isNewObject)
			{
				ManageNewObjectPointers(objectInfo, classInfo, position, metaModel);
			}
			if (NeoDatis.Odb.OdbConfiguration.SaveHistory())
			{
				classInfo.AddHistory(new NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.History.InsertHistoryInfo
					("insert", oid, position, objectInfo.GetPreviousObjectOID(), objectInfo.GetNextObjectOID
					()));
			}
			fsi.SetWritePosition(position, writeDataInTransaction);
			objectInfo.SetPosition(position);
			// Block size
			fsi.WriteInt(0, writeDataInTransaction, "block size");
			// Block type
			fsi.WriteByte(NeoDatis.Odb.Impl.Core.Layers.Layer3.Block.BlockTypes.BlockTypeNonNativeObject
				, writeDataInTransaction, "object block type");
			// The object id
			fsi.WriteLong(oid.GetObjectId(), writeDataInTransaction, "oid", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
				.DataWriteAction);
			// Class info id
			fsi.WriteLong(classInfo.GetId().GetObjectId(), writeDataInTransaction, "class info id"
				, NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction.DataWriteAction);
			// previous instance
			WriteOid(objectInfo.GetPreviousObjectOID(), writeDataInTransaction, "prev instance"
				, NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction.DataWriteAction);
			// next instance
			WriteOid(objectInfo.GetNextObjectOID(), writeDataInTransaction, "next instance", 
				NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction.DataWriteAction);
			// creation date, for update operation must be the original one
			fsi.WriteLong(objectInfo.GetHeader().GetCreationDate(), writeDataInTransaction, "creation date"
				, NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction.DataWriteAction);
			fsi.WriteLong(NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs(), writeDataInTransaction
				, "update date", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction.DataWriteAction
				);
			// TODO check next version number
			fsi.WriteInt(objectInfo.GetHeader().GetObjectVersion(), writeDataInTransaction, "object version number"
				);
			// not used yet. But it will point to an internal object of type
			// ObjectReference that will have details on the references:
			// All the objects that point to it: to enable object integrity
			fsi.WriteLong(-1, writeDataInTransaction, "object reference pointer", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
				.DataWriteAction);
			// True if this object have been synchronized with main database, else
			// false
			fsi.WriteBoolean(false, writeDataInTransaction, "is syncronized with external db"
				);
			int nbAttributes = objectInfo.GetClassInfo().GetAttributes().Count;
			// now write the number of attributes and the position of all
			// attributes, we do not know them yet, so write 00 but at the end
			// of the write operation
			// These positions will be updated
			// The positions that is going to be written are 'int' representing
			// the offset position of the attribute
			// first write the number of attributes
			fsi.WriteInt(nbAttributes, writeDataInTransaction, "nb attr");
			// Store the position
			long attributePositionStart = fsi.GetPosition();
			// TODO Could remove this, and pull to the right position
			for (int i = 0; i < nbAttributes; i++)
			{
				fsi.WriteInt(0, writeDataInTransaction, "attr id -1");
				fsi.WriteLong(0, writeDataInTransaction, "att pos", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
					.DataWriteAction);
			}
			long[] attributesIdentification = new long[nbAttributes];
			int[] attributeIds = new int[nbAttributes];
			// Puts the object info in the cache
			// storageEngine.getSession().getCache().addObject(position,
			// aoi.getObject(), objectInfo.getHeader());
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassAttributeInfo cai = null;
			NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo aoi2 = null;
			long nativeAttributePosition = -1;
			NeoDatis.Odb.OID nonNativeAttributeOid = null;
			long maxWritePosition = fsi.GetPosition();
			// Loop on all attributes
			for (int i = 0; i < nbAttributes; i++)
			{
				// Gets the attribute meta description
				cai = classInfo.GetAttributeInfo(i);
				// Gets the id of the attribute
				attributeIds[i] = cai.GetId();
				// Gets the attribute data
				aoi2 = objectInfo.GetAttributeValueFromId(cai.GetId());
				if (aoi2 == null)
				{
					// This only happens in 1 case : when a class has a field with
					// the same name of one of is superclass. In this, the deeper
					// attribute is null
					if (cai.IsNative())
					{
						aoi2 = new NeoDatis.Odb.Core.Layers.Layer2.Meta.NullNativeObjectInfo(cai.GetAttributeType
							().GetId());
					}
					else
					{
						aoi2 = new NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeNullObjectInfo(cai.GetClassInfo
							());
					}
				}
				if (aoi2.IsNative())
				{
					nativeAttributePosition = InternalStoreObject((NeoDatis.Odb.Core.Layers.Layer2.Meta.NativeObjectInfo
						)aoi2);
					// For native objects , odb stores their position
					attributesIdentification[i] = nativeAttributePosition;
				}
				else
				{
					if (aoi2.IsObjectReference())
					{
						NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectReference or = (NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectReference
							)aoi2;
						nonNativeAttributeOid = or.GetOid();
					}
					else
					{
						nonNativeAttributeOid = StoreObject(null, (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
							)aoi2);
					}
					// For non native objects , odb stores its oid as a negative
					// number!!u
					if (nonNativeAttributeOid != null)
					{
						attributesIdentification[i] = -nonNativeAttributeOid.GetObjectId();
					}
					else
					{
						attributesIdentification[i] = NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
							.NullObjectIdId;
					}
				}
				long p = fsi.GetPosition();
				if (p > maxWritePosition)
				{
					maxWritePosition = p;
				}
			}
			// Updates attributes identification in the object info header
			objectInfo.GetHeader().SetAttributesIdentification(attributesIdentification);
			objectInfo.GetHeader().SetAttributesIds(attributeIds);
			long positionAfterWrite = maxWritePosition;
			// Now writes back the attribute positions
			fsi.SetWritePosition(attributePositionStart, writeDataInTransaction);
			for (int i = 0; i < attributesIdentification.Length; i++)
			{
				fsi.WriteInt(attributeIds[i], writeDataInTransaction, "attr id");
				fsi.WriteLong(attributesIdentification[i], writeDataInTransaction, "att real pos"
					, NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction.DataWriteAction);
				// if (classInfo.getAttributeInfo(i).isNonNative() &&
				// attributesIdentification[i] > 0) {
				if (objectInfo.GetAttributeValueFromId(attributeIds[i]).IsNonNativeObject() && attributesIdentification
					[i] > 0)
				{
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.NonNativeAttributeStoredByPositionInsteadOfOid
						.AddParameter(classInfo.GetAttributeInfo(i).GetName()).AddParameter(classInfo.GetFullClassName
						()).AddParameter(attributesIdentification[i]));
				}
			}
			fsi.SetWritePosition(positionAfterWrite, writeDataInTransaction);
			int blockSize = (int)(positionAfterWrite - position);
			try
			{
				WriteBlockSizeAt(position, blockSize, writeDataInTransaction, objectInfo);
			}
			catch (NeoDatis.Odb.ODBRuntimeException e)
			{
				NeoDatis.Tool.DLogger.Debug("Error while writing block size. pos after write " + 
					positionAfterWrite + " / start pos = " + position);
				// throw new ODBRuntimeException(storageEngine,"Error while writing
				// block size. pos after write " + positionAfterWrite + " / start
				// pos = " + position,e);
				throw;
			}
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
			{
				NeoDatis.Tool.DLogger.Debug(DepthToSpaces() + "  Attributes positions of object with oid "
					 + oid + " are " + NeoDatis.Tool.DisplayUtility.LongArrayToString(attributesIdentification
					));
				NeoDatis.Tool.DLogger.Debug(DepthToSpaces() + "End Writing non native object at "
					 + position + " with oid " + oid + " - prev oid=" + objectInfo.GetPreviousObjectOID
					() + " / next oid=" + objectInfo.GetNextObjectOID());
				if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogIdDebug))
				{
					NeoDatis.Tool.DLogger.Debug(" - current buffer : " + fsi.GetIo().ToString());
				}
			}
			// Only insert in index for new objects
			if (isNewObject)
			{
				// insert object id in indexes, if exist
				ManageIndexesForInsert(oid, objectInfo);
				if (hasObject)
				{
					triggerManager.ManageInsertTriggerAfter(objectInfo.GetClassInfo().GetFullClassName
						(), objectInfo.GetObject(), oid);
				}
				else
				{
					// triggers
					triggerManager.ManageInsertTriggerAfter(objectInfo.GetClassInfo().GetFullClassName
						(), objectInfo, oid);
				}
			}
			return oid;
		}

		/// <summary>Updates pointers of objects, Only changes uncommitted info pointers</summary>
		/// <param name="objectInfo">The meta representation of the object being inserted</param>
		/// <param name="classInfo">The class of the object being inserted</param>
		/// <param name="position">The position where the object is being inserted @</param>
		private void ManageNewObjectPointers(NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 objectInfo, NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo classInfo, long position
			, NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel metaModel)
		{
			NeoDatis.Odb.Core.Transaction.ICache cache = storageEngine.GetSession(true).GetCache
				();
			bool isFirstUncommitedObject = !classInfo.GetUncommittedZoneInfo().HasObjects();
			// if it is the first uncommitted object
			if (isFirstUncommitedObject)
			{
				classInfo.GetUncommittedZoneInfo().first = objectInfo.GetOid();
				NeoDatis.Odb.OID lastCommittedObjectOid = classInfo.GetCommitedZoneInfo().last;
				if (lastCommittedObjectOid != null)
				{
					// Also updates the last committed object next object oid in
					// memory to connect the committed
					// zone with unconnected for THIS transaction (only in memory)
					NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader oih = cache.GetObjectInfoHeaderFromOid
						(lastCommittedObjectOid, true);
					oih.SetNextObjectOID(objectInfo.GetOid());
					// And sets the previous oid of the current object with the last
					// committed oid
					objectInfo.SetPreviousInstanceOID(lastCommittedObjectOid);
				}
			}
			else
			{
				// Gets the last object, updates its (next object)
				// pointer to the new object and updates the class info 'last
				// uncommitted object
				// oid' field
				NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader oip = classInfo.GetLastObjectInfoHeader
					();
				if (oip == null)
				{
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.InternalError
						.AddParameter("last OIP is null in manageNewObjectPointers oid=" + objectInfo.GetOid
						()));
				}
				if (oip.GetNextObjectOID() != objectInfo.GetOid())
				{
					oip.SetNextObjectOID(objectInfo.GetOid());
					// Here we are working in unconnected zone, so this
					// can be done without transaction: actually
					// write in database file
					UpdateNextObjectFieldOfObjectInfo(oip.GetOid(), oip.GetNextObjectOID(), false);
					objectInfo.SetPreviousInstanceOID(oip.GetOid());
					// Resets the class info oid: In some case,
					// (client // server) it may be -1.
					oip.SetClassInfoId(classInfo.GetId());
					// object info oip has been changed, we must put it
					// in the cache to turn this change available for current
					// transaction until the commit
					storageEngine.GetSession(true).GetCache().AddObjectInfo(oip);
				}
			}
			// always set the new last object oid and the number of objects
			classInfo.GetUncommittedZoneInfo().last = objectInfo.GetOid();
			classInfo.GetUncommittedZoneInfo().IncreaseNbObjects();
			// Then updates the last info pointers of the class info
			// with this new created object
			// At this moment, the objectInfo.getHeader() do not have the
			// attribute ids.
			// but later in this code, the attributes will be set, so the class
			// info also will have them
			classInfo.SetLastObjectInfoHeader(objectInfo.GetHeader());
			// // Saves the fact that something has changed in the class (number of
			// objects and/or last object oid)
			storageEngine.GetSession(true).GetMetaModel().AddChangedClass(classInfo);
		}

		/// <summary>Insert the object in the index</summary>
		/// <param name="oid">The object id</param>
		/// <param name="nnoi">The object meta represenation</param>
		/// <returns>The number of indexes</returns>
		public virtual int ManageIndexesForInsert(NeoDatis.Odb.OID oid, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 nnoi)
		{
			NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoIndex
				> indexes = nnoi.GetClassInfo().GetIndexes();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoIndex index = null;
			for (int i = 0; i < indexes.Count; i++)
			{
				index = indexes[i];
				try
				{
					index.GetBTree().Insert(index.ComputeKey(nnoi), oid);
				}
				catch (NeoDatis.Btree.Exception.DuplicatedKeyException e)
				{
					// rollback what has been done
					// bug #2510966
					GetSession().Rollback();
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.DuplicatedKeyInIndex
						.AddParameter(index.GetName()).AddParameter(e.Message));
				}
				// Check consistency : index should have size equal to the class
				// info element number
				if (index.GetBTree().GetSize() != nnoi.GetClassInfo().GetNumberOfObjects())
				{
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.BtreeSizeDiffersFromClassElementNumber
						.AddParameter(index.GetBTree().GetSize()).AddParameter(nnoi.GetClassInfo().GetNumberOfObjects
						()));
				}
			}
			return indexes.Count;
		}

		/// <summary>Insert the object in the index</summary>
		/// <param name="oid">The object id</param>
		/// <param name="nnoi">The object meta represenation</param>
		/// <returns>The number of indexes</returns>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual int ManageIndexesForDelete(NeoDatis.Odb.OID oid, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 nnoi)
		{
			NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoIndex
				> indexes = nnoi.GetClassInfo().GetIndexes();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoIndex index = null;
			for (int i = 0; i < indexes.Count; i++)
			{
				index = indexes[i];
				// TODO manage collision!
				index.GetBTree().Delete(index.ComputeKey(nnoi), oid);
				// Check consistency : index should have size equal to the class
				// info element number
				if (index.GetBTree().GetSize() != nnoi.GetClassInfo().GetNumberOfObjects())
				{
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.BtreeSizeDiffersFromClassElementNumber
						.AddParameter(index.GetBTree().GetSize()).AddParameter(nnoi.GetClassInfo().GetNumberOfObjects
						()));
				}
			}
			return indexes.Count;
		}

		public virtual int ManageIndexesForUpdate(NeoDatis.Odb.OID oid, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 nnoi, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo oldMetaRepresentation
			)
		{
			// takes the indexes from the oldMetaRepresentation because noi comes
			// from the client and is not always
			// in sync with the server meta model (In Client Server mode)
			NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoIndex
				> indexes = oldMetaRepresentation.GetClassInfo().GetIndexes();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoIndex index = null;
			NeoDatis.Tool.Wrappers.OdbComparable oldKey = null;
			NeoDatis.Tool.Wrappers.OdbComparable newKey = null;
			for (int i = 0; i < indexes.Count; i++)
			{
				index = indexes[i];
				oldKey = index.ComputeKey(oldMetaRepresentation);
				newKey = index.ComputeKey(nnoi);
				// Only update index if key has changed!
				if (oldKey.CompareTo(newKey) != 0)
				{
					NeoDatis.Btree.IBTree btree = index.GetBTree();
					// TODO manage collision!
					object old = btree.Delete(oldKey, oid);
					// TODO check if old is equal to oldKey
					btree.Insert(newKey, oid);
					// Check consistency : index should have size equal to the class
					// info element number
					if (index.GetBTree().GetSize() != nnoi.GetClassInfo().GetNumberOfObjects())
					{
						throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.BtreeSizeDiffersFromClassElementNumber
							.AddParameter(index.GetBTree().GetSize()).AddParameter(nnoi.GetClassInfo().GetNumberOfObjects
							()));
					}
				}
			}
			return indexes.Count;
		}

		/// <param name="oid">The Oid of the object to be inserted</param>
		/// <param name="nnoi">
		/// The object meta representation The object to be inserted in
		/// the database
		/// </param>
		/// <param name="isNewObject">To indicate if object is new</param>
		/// <returns>The position of the inserted object</returns>
		public virtual NeoDatis.Odb.OID InsertNonNativeObject(NeoDatis.Odb.OID oid, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 nnoi, bool isNewObject)
		{
			try
			{
				NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci = nnoi.GetClassInfo();
				object @object = nnoi.GetObject();
				// First check if object is already being inserted
				// This method returns -1 if object is not being inserted
				NeoDatis.Odb.OID cachedOid = GetSession().GetCache().IdOfInsertingObject(@object);
				if (cachedOid != null)
				{
					return cachedOid;
				}
				// Then checks if the class of this object already exist in the
				// meta model
				ci = AddClass(ci, true);
				// Resets the ClassInfo in the objectInfo to be sure it contains all
				// updated class info data
				nnoi.SetClassInfo(ci);
				// Mark this object as being inserted. To manage cyclic relations
				// The oid may be equal to -1
				// Later in the process the cache will be updated with the right oid
				GetSession().GetCache().StartInsertingObjectWithOid(@object, oid, nnoi);
				// false : do not write data in transaction. Data are always written
				// directly to disk. Pointers are written in transaction
				NeoDatis.Odb.OID newOid = WriteNonNativeObjectInfo(oid, nnoi, -1, false, isNewObject
					);
				if (newOid != NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant.NullObjectId)
				{
					GetSession().GetCache().AddObject(newOid, @object, nnoi.GetHeader());
				}
				return newOid;
			}
			finally
			{
			}
		}

		// This will be done by the mainStoreObject method
		// Context.getCache().endInsertingObject(object);
		/// <param name="noi">
		/// The native object meta representation The object to be
		/// inserted in the database
		/// </param>
		/// <returns>The position of the inserted object</returns>
		private long InsertNativeObject(NeoDatis.Odb.Core.Layers.Layer2.Meta.NativeObjectInfo
			 noi)
		{
			long writePosition = fsi.GetAvailablePosition();
			fsi.SetWritePosition(writePosition, true);
			// true,false = update pointers,do not write in transaction, writes
			// directly to hard disk
			long position = WriteNativeObjectInfo(noi, writePosition, true, false);
			return position;
		}

		/// <summary>
		/// Store a meta representation of an object(already as meta
		/// representation)in ODBFactory database.
		/// </summary>
		/// <remarks>
		/// Store a meta representation of an object(already as meta
		/// representation)in ODBFactory database.
		/// To detect if object must be updated or insert, we use the cache. To
		/// update an object, it must be first selected from the database. When an
		/// object is to be stored, if it exist in the cache, then it will be
		/// updated, else it will be inserted as a new object. If the object is null,
		/// the cache will be used to check if the meta representation is in the
		/// cache
		/// </remarks>
		/// <param name="oid">The oid of the object to be inserted/updates</param>
		/// <param name="nnoi">The meta representation of an object</param>
		/// <returns>The object position</returns>
		public virtual NeoDatis.Odb.OID StoreObject(NeoDatis.Odb.OID oid, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 nnoi)
		{
			// first detects if we must perform an insert or an update
			// If object is in the cache, we must perform an update, else an insert
			object @object = nnoi.GetObject();
			bool mustUpdate = false;
			NeoDatis.Odb.Core.Transaction.ICache cache = GetSession().GetCache();
			if (@object != null)
			{
				NeoDatis.Odb.OID cacheOid = cache.IdOfInsertingObject(@object);
				if (cacheOid != null)
				{
					return cacheOid;
				}
				// throw new ODBRuntimeException("Inserting meta representation of
				// an object without the object itself is not yet supported");
				mustUpdate = cache.ExistObject(@object);
			}
			if (!mustUpdate)
			{
				mustUpdate = nnoi.GetOid() != NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
					.NullObjectId;
			}
			// To enable auto - reconnect object loaded from previous sessions
			// auto reconnect is on
			if (!mustUpdate && NeoDatis.Odb.OdbConfiguration.ReconnectObjectsToSession())
			{
				NeoDatis.Odb.Core.Transaction.ICrossSessionCache crossSessionCache = NeoDatis.Odb.Impl.Core.Transaction.CacheFactory
					.GetCrossSessionCache(storageEngine.GetBaseIdentification().GetIdentification());
				if (crossSessionCache.ExistObject(@object))
				{
					storageEngine.Reconnect(@object);
					mustUpdate = true;
				}
			}
			if (mustUpdate)
			{
				return UpdateNonNativeObjectInfo(nnoi, false);
			}
			return InsertNonNativeObject(oid, nnoi, true);
		}

		/// <summary>
		/// Store a meta representation of a native object(already as meta
		/// representation)in ODBFactory database.
		/// </summary>
		/// <remarks>
		/// Store a meta representation of a native object(already as meta
		/// representation)in ODBFactory database. A Native object is an object that
		/// use native language type, String for example
		/// To detect if object must be updated or insert, we use the cache. To
		/// update an object, it must be first selected from the database. When an
		/// object is to be stored, if it exist in the cache, then it will be
		/// updated, else it will be inserted as a new object. If the object is null,
		/// the cache will be used to check if the meta representation is in the
		/// cache
		/// </remarks>
		/// <param name="nnoi">The meta representation of an object</param>
		/// <returns>The object position @</returns>
		internal virtual long InternalStoreObject(NeoDatis.Odb.Core.Layers.Layer2.Meta.NativeObjectInfo
			 noi)
		{
			return InsertNativeObject(noi);
		}

		public virtual NeoDatis.Odb.OID UpdateObject(NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
			 aoi, bool forceUpdate)
		{
			if (aoi.IsNonNativeObject())
			{
				return UpdateNonNativeObjectInfo((NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
					)aoi, forceUpdate);
			}
			if (aoi.IsNative())
			{
				return UpdateObject(aoi, forceUpdate);
			}
			// TODO : here should use if then else
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.AbstractObjectInfoTypeNotSupported
				.AddParameter(aoi.GetType().FullName));
		}

		/// <summary>Updates an object.</summary>
		/// <remarks>
		/// Updates an object.
		/// <pre>
		/// Try to update in place. Only change what has changed. This is restricted to particular types (fixed size types). If in place update is
		/// not possible, then deletes the current object and creates a new at the end of the database file and updates
		/// OID object position.
		/// &#064;param object The object to be updated
		/// &#064;param forceUpdate when true, no verification is done to check if update must be done.
		/// &#064;return The oid of the object, as a negative number
		/// &#064;
		/// </remarks>
		public virtual NeoDatis.Odb.OID UpdateNonNativeObjectInfo(NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 nnoi, bool forceUpdate)
		{
			nbCallsToUpdate++;
			bool hasObject = true;
			string message = null;
			object @object = nnoi.GetObject();
			NeoDatis.Odb.OID oid = nnoi.GetOid();
			if (@object == null)
			{
				hasObject = false;
			}
			// When there is index,we must *always* load the old meta representation
			// to compute index keys
			bool withIndex = !nnoi.GetClassInfo().GetIndexes().IsEmpty();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo oldMetaRepresentation = 
				null;
			// Used to check consistency, at the end, the number of
			// nbConnectedObjects must and nbUnconnected must remain unchanged
			long nbConnectedObjects = nnoi.GetClassInfo().GetCommitedZoneInfo().GetNbObjects(
				);
			long nbNonConnectedObjects = nnoi.GetClassInfo().GetUncommittedZoneInfo().GetNbObjects
				();
			bool objectHasChanged = false;
			try
			{
				NeoDatis.Odb.Core.Transaction.ISession lsession = GetSession();
				long positionBeforeWrite = fsi.GetPosition();
				NeoDatis.Odb.Core.Transaction.ITmpCache tmpCache = lsession.GetTmpCache();
				NeoDatis.Odb.Core.Transaction.ICache cache = lsession.GetCache();
				// Get header of the object (position, previous object position,
				// next
				// object position and class info position)
				// The header must be in the cache.
				NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader lastHeader = cache.GetObjectInfoHeaderFromOid
					(oid, true);
				if (lastHeader == null)
				{
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.UnexpectedSituation
						.AddParameter("Header is null in update"));
				}
				if (lastHeader.GetOid() == null)
				{
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.InternalError
						.AddParameter("Header oid is null for oid " + oid));
				}
				bool objectIsInConnectedZone = cache.ObjectWithIdIsInCommitedZone(oid);
				long currentPosition = lastHeader.GetPosition();
				// When using client server mode, we must re-read the position of
				// the object with oid. Because, another session may
				// have updated the object, and in this case, the position of the
				// object in the cache may be invalid
				// TODO It should be done only when the object has been deleted or
				// updated by another session. Should check this
				// Doing this with new objects (created in the current session, the
				// last committed
				// object position will be negative, in this case we must use the
				// currentPosition
				if (!isLocalMode)
				{
					long lastCommitedObjectPosition = idManager.GetObjectPositionWithOid(oid, false);
					if (lastCommitedObjectPosition > 0)
					{
						currentPosition = lastCommitedObjectPosition;
					}
					// Some infos that come from the client are not set
					// So we overwrite them here : example : object version. Update
					// date is not important here
					// Because, as we are updating the object, the update date will
					// be updated too
					nnoi.GetHeader().SetObjectVersion(lastHeader.GetObjectVersion());
					nnoi.GetHeader().SetUpdateDate(lastHeader.GetUpdateDate());
				}
				// for client server
				if (nnoi.GetPosition() == -1)
				{
					nnoi.GetHeader().SetPosition(currentPosition);
				}
				if (currentPosition == -1)
				{
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.InstancePositionIsNegative
						.AddParameter(currentPosition).AddParameter(oid).AddParameter("In Object Info Header"
						));
				}
				if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
				{
					message = DepthToSpaces() + "start updating object at " + currentPosition + ", oid="
						 + oid + " : " + (nnoi != null ? nnoi.ToString() : "null");
					NeoDatis.Tool.DLogger.Debug(message);
				}
				// triggers,FIXME passing null to old object representation
				if (hasObject)
				{
					storageEngine.GetTriggerManager().ManageUpdateTriggerBefore(nnoi.GetClassInfo().GetFullClassName
						(), null, @object, oid);
				}
				else
				{
					storageEngine.GetTriggerManager().ManageUpdateTriggerBefore(nnoi.GetClassInfo().GetFullClassName
						(), null, nnoi, oid);
				}
				// Use to control if the in place update is ok. The
				// ObjectInstrospector stores the number of changes
				// that were detected and here we try to apply them using in place
				// update.If at the end
				// of the in place update the number of applied changes is smaller
				// then the number
				// of detected changes, then in place update was not successfully,
				// we
				// must do a real update,
				// creating an object elsewhere :-(
				int nbAppliedChanges = 0;
				if (!forceUpdate)
				{
					NeoDatis.Odb.OID cachedOid = cache.IdOfInsertingObject(@object);
					if (cachedOid != null)
					{
						// The object is being inserted (must be a cyclic
						// reference), simply returns id id
						return cachedOid;
					}
					// the nnoi (NonNativeObjectInfo is the meta representation of
					// the object to update
					// To know what must be upated we must get the meta
					// representation of this object before
					// The modification. Taking this 'old' meta representation from
					// the
					// cache does not resolve
					// : because cache is a reference to the real object and object
					// has been changed,
					// so the cache is pointing to the reference, that has changed!
					// This old meta representation must be re-read from the last
					// committed database
					// false, = returnInstance (java object) = false
					try
					{
						bool useCache = !objectIsInConnectedZone;
						oldMetaRepresentation = objectReader.ReadNonNativeObjectInfoFromPosition(null, oid
							, currentPosition, useCache, false);
						tmpCache.ClearObjectInfos();
					}
					catch (NeoDatis.Odb.ODBRuntimeException e)
					{
						throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.InternalError
							.AddParameter("Error while reading old Object Info of oid " + oid + " at pos " +
							 currentPosition), e);
					}
					// Make sure we work with the last version of the object
					int onDiskVersion = oldMetaRepresentation.GetHeader().GetObjectVersion();
					long onDiskUpdateDate = oldMetaRepresentation.GetHeader().GetUpdateDate();
					int inCacheVersion = lastHeader.GetObjectVersion();
					long inCacheUpdateDate = lastHeader.GetUpdateDate();
					if (onDiskUpdateDate > inCacheUpdateDate || onDiskVersion > inCacheVersion)
					{
						lastHeader = oldMetaRepresentation.GetHeader();
					}
					nnoi.SetHeader(lastHeader);
					// increase the object version number from the old meta
					// representation
					nnoi.GetHeader().IncrementVersionAndUpdateDate();
					// Keep the creation date
					nnoi.GetHeader().SetCreationDate(oldMetaRepresentation.GetHeader().GetCreationDate
						());
					// Set the object of the old meta to make the object comparator
					// understand, they are 2
					// meta representation of the same object
					// TODO , check if if is the best way to do
					oldMetaRepresentation.SetObject(nnoi.GetObject());
					// Reset the comparator
					comparator.Clear();
					objectHasChanged = comparator.HasChanged(oldMetaRepresentation, nnoi);
					if (!objectHasChanged)
					{
						fsi.SetWritePosition(positionBeforeWrite, true);
						if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
						{
							NeoDatis.Tool.DLogger.Debug(DepthToSpaces() + "updateObject : Object is unchanged - doing nothing"
								);
						}
						return oid;
					}
					if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
					{
						NeoDatis.Tool.DLogger.Debug(DepthToSpaces() + "\tmax recursion level is " + comparator
							.GetMaxObjectRecursionLevel());
						NeoDatis.Tool.DLogger.Debug(DepthToSpaces() + "\tattribute actions are : " + comparator
							.GetChangedAttributeActions());
						NeoDatis.Tool.DLogger.Debug(DepthToSpaces() + "\tnew objects are : " + comparator
							.GetNewObjects());
					}
					if (NeoDatis.Odb.OdbConfiguration.InPlaceUpdate() && comparator.SupportInPlaceUpdate
						())
					{
						nbAppliedChanges = ManageInPlaceUpdate(comparator, @object, oid, lastHeader, cache
							, objectIsInConnectedZone);
						// if number of applied changes is equal to the number of
						// detected change
						if (nbAppliedChanges == comparator.GetNbChanges())
						{
							nbInPlaceUpdates++;
							UpdateUpdateTimeAndObjectVersionNumber(lastHeader, true);
							cache.AddObject(oid, @object, lastHeader);
							return oid;
						}
					}
				}
				// If we reach this update, In Place Update was not possible. Do a
				// normal update. Deletes the
				// current object and creates a new one
				if (oldMetaRepresentation == null && withIndex)
				{
					// We must load old meta representation to be able to compute
					// old index key to update index
					oldMetaRepresentation = objectReader.ReadNonNativeObjectInfoFromPosition(null, oid
						, currentPosition, false, false);
				}
				nbNormalUpdates++;
				if (hasObject)
				{
					cache.StartInsertingObjectWithOid(@object, oid, nnoi);
				}
				// gets class info from in memory meta model
				NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci = lsession.GetMetaModel().GetClassInfoFromId
					(lastHeader.GetClassInfoId());
				if (hasObject)
				{
					// removes the object from the cache
					// cache.removeObjectWithOid(oid, object);
					cache.EndInsertingObject(@object);
				}
				NeoDatis.Odb.OID previousObjectOID = lastHeader.GetPreviousObjectOID();
				NeoDatis.Odb.OID nextObjectOid = lastHeader.GetNextObjectOID();
				if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
				{
					NeoDatis.Tool.DLogger.Debug(DepthToSpaces() + "Updating object " + nnoi.ToString(
						));
					NeoDatis.Tool.DLogger.Debug(DepthToSpaces() + "position =  " + currentPosition + 
						" | prev instance = " + previousObjectOID + " | next instance = " + nextObjectOid
						);
				}
				nnoi.SetPreviousInstanceOID(previousObjectOID);
				nnoi.SetNextObjectOID(nextObjectOid);
				// Mark the block of current object as deleted
				MarkAsDeleted(currentPosition, oid, objectIsInConnectedZone);
				// Creates the new object
				oid = InsertNonNativeObject(oid, nnoi, false);
				// This position after write must be call just after the insert!!
				long positionAfterWrite = fsi.GetPosition();
				if (hasObject)
				{
					// update cache
					cache.AddObject(oid, @object, nnoi.GetHeader());
				}
				//TODO check if we must update cross session cache
				fsi.SetWritePosition(positionAfterWrite, true);
				long nbConnectedObjectsAfter = nnoi.GetClassInfo().GetCommitedZoneInfo().GetNbObjects
					();
				long nbNonConnectedObjectsAfter = nnoi.GetClassInfo().GetUncommittedZoneInfo().GetNbObjects
					();
				if (nbConnectedObjectsAfter != nbConnectedObjects || nbNonConnectedObjectsAfter !=
					 nbNonConnectedObjects)
				{
				}
				// TODO check this
				// throw new
				// ODBRuntimeException(Error.INTERNAL_ERROR.addParameter("Error
				// in nb connected/unconnected counter"));
				return oid;
			}
			catch (System.Exception e)
			{
				message = DepthToSpaces() + "Error updating object " + nnoi.ToString() + " : " + 
					NeoDatis.Tool.Wrappers.OdbString.ExceptionToString(e, true);
				NeoDatis.Tool.DLogger.Error(message);
				throw new NeoDatis.Odb.ODBRuntimeException(e, message);
			}
			finally
			{
				if (objectHasChanged)
				{
					if (withIndex)
					{
						ManageIndexesForUpdate(oid, nnoi, oldMetaRepresentation);
					}
					// triggers,FIXME passing null to old object representation
					// (oldMetaRepresentation may be null)
					if (hasObject)
					{
						storageEngine.GetTriggerManager().ManageUpdateTriggerAfter(nnoi.GetClassInfo().GetFullClassName
							(), oldMetaRepresentation, @object, oid);
					}
					else
					{
						storageEngine.GetTriggerManager().ManageUpdateTriggerAfter(nnoi.GetClassInfo().GetFullClassName
							(), oldMetaRepresentation, nnoi, oid);
					}
				}
				if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
				{
					NeoDatis.Tool.DLogger.Debug(DepthToSpaces() + "end updating object with oid=" + oid
						 + " at pos " + nnoi.GetPosition() + " => " + nnoi.ToString());
				}
			}
		}

		/// <summary>Upate the version number of the object</summary>
		/// <param name="header"></param>
		/// <param name="writeInTransaction"></param>
		private void UpdateUpdateTimeAndObjectVersionNumber(NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader
			 header, bool writeInTransaction)
		{
			long objectPosition = header.GetPosition();
			fsi.SetWritePosition(objectPosition + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.ObjectOffsetUpdateDate, writeInTransaction);
			fsi.WriteLong(header.GetUpdateDate(), writeInTransaction, "update date time", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
				.DataWriteAction);
			fsi.WriteInt(header.GetObjectVersion(), writeInTransaction, "object version");
		}

		protected virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader GetObjectInfoHeader
			(NeoDatis.Odb.OID oid, NeoDatis.Odb.Core.Transaction.ICache cache)
		{
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader oih = cache.GetObjectInfoHeaderFromOid
				(oid, false);
			// If object is not in the cache, then read the header from the file
			if (oih == null)
			{
				oih = objectReader.ReadObjectInfoHeaderFromOid(oid, false);
			}
			return oih;
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader UpdateNextObjectPreviousPointersInCache
			(NeoDatis.Odb.OID nextObjectOID, NeoDatis.Odb.OID previousObjectOID, NeoDatis.Odb.Core.Transaction.ICache
			 cache)
		{
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader oip = cache.GetObjectInfoHeaderFromOid
				(nextObjectOID, false);
			// If object is not in the cache, then read the header from the file
			if (oip == null)
			{
				oip = objectReader.ReadObjectInfoHeaderFromOid(nextObjectOID, false);
				cache.AddObjectInfo(oip);
			}
			oip.SetPreviousObjectOID(previousObjectOID);
			return oip;
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader UpdatePreviousObjectNextPointersInCache
			(NeoDatis.Odb.OID nextObjectOID, NeoDatis.Odb.OID previousObjectOID, NeoDatis.Odb.Core.Transaction.ICache
			 cache)
		{
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader oip = cache.GetObjectInfoHeaderFromOid
				(previousObjectOID, false);
			// If object is not in the cache, then read the header from the file
			if (oip == null)
			{
				oip = objectReader.ReadObjectInfoHeaderFromOid(previousObjectOID, false);
				cache.AddObjectInfo(oip);
			}
			oip.SetNextObjectOID(nextObjectOID);
			return oip;
		}

		/// <summary>Manage in place update.</summary>
		/// <remarks>
		/// Manage in place update. Just write the value at the exact position if
		/// possible.
		/// </remarks>
		/// <param name="objectComparator">
		/// Contains all infos about differences between all version
		/// objects and new version
		/// </param>
		/// <param name="@object">The object being modified (new version)</param>
		/// <param name="oid">The oid of the object being modified</param>
		/// <param name="header">
		/// The header of the object meta representation (Comes from the
		/// cache)
		/// </param>
		/// <param name="cache">The cache it self</param>
		/// <param name="objectInInConnectedZone">
		/// A boolean value to indicate if object is in connected zone. I
		/// true, change must be made in transaction. If false, changes
		/// can be made in the database file directly.
		/// </param>
		/// <returns>The number of in place update successfully executed</returns>
		/// <exception cref="System.Exception">System.Exception</exception>
		private int ManageInPlaceUpdate(NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.IObjectInfoComparator
			 objectComparator, object @object, NeoDatis.Odb.OID oid, NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader
			 header, NeoDatis.Odb.Core.Transaction.ICache cache, bool objectIsInConnectedZone
			)
		{
			bool canUpdateInPlace = true;
			// If object is is connected zone, changes must be done in transaction,
			// if not in connected zone, changes can be made out of
			// transaction, directly to the database
			bool writeInTransaction = objectIsInConnectedZone;
			int nbAppliedChanges = 0;
			// if 0, only direct attribute have been changed
			// if (objectComparator.getMaxObjectRecursionLevel() == 0) {
			// if some direct native attribute have changed
			if (objectComparator.GetChangedAttributeActions().Count > 0)
			{
				NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.ChangedNativeAttributeAction caa = null;
				// Check if in place update is possible
				System.Collections.Generic.IList<NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.ChangedAttribute
					> actions = objectComparator.GetChangedAttributeActions();
				for (int i = 0; i < actions.Count; i++)
				{
					if (actions[i] is NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.ChangedNativeAttributeAction)
					{
						caa = (NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.ChangedNativeAttributeAction)
							actions[i];
						if (caa.ReallyCantDoInPlaceUpdate())
						{
							canUpdateInPlace = false;
							break;
						}
						if (false && !caa.InPlaceUpdateIsGuaranteed())
						{
							if (caa.IsString() && caa.GetUpdatePosition() != NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
								.NullObjectPosition)
							{
								long position = SafeOverWriteAtomicNativeObject(caa.GetUpdatePosition(), (NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo
									)caa.GetNoiWithNewValue(), writeInTransaction);
								canUpdateInPlace = position != -1;
								if (!canUpdateInPlace)
								{
									break;
								}
							}
							else
							{
								canUpdateInPlace = false;
								break;
							}
						}
						else
						{
							fsi.SetWritePosition(caa.GetUpdatePosition(), true);
							WriteAtomicNativeObject((NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo
								)caa.GetNoiWithNewValue(), writeInTransaction);
						}
					}
					else
					{
						if (actions[i] is NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.ChangedObjectReferenceAttributeAction)
						{
							NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.ChangedObjectReferenceAttributeAction
								 coraa = (NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.ChangedObjectReferenceAttributeAction
								)actions[i];
							UpdateObjectReference(coraa.GetUpdatePosition(), coraa.GetNewId(), writeInTransaction
								);
						}
					}
					nbAppliedChanges++;
				}
				if (canUpdateInPlace)
				{
					if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
					{
						NeoDatis.Tool.DLogger.Debug(DepthToSpaces() + "Sucessfull in place updating");
					}
				}
			}
			// if canUpdateInplace is false, a full update (writing
			// object elsewhere) is necessary so
			// there is no need to try to update object references.
			if (canUpdateInPlace)
			{
				NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.NewNonNativeObjectAction nnnoa = null;
				// For non native attribute that have been replaced!
				for (int i = 0; i < objectComparator.GetNewObjectMetaRepresentations().Count; i++)
				{
					// to avoid stackOverFlow, check if the object is
					// already beeing inserted
					nnnoa = objectComparator.GetNewObjectMetaRepresentation(i);
					if (cache.IdOfInsertingObject(nnnoa) == null)
					{
						NeoDatis.Odb.OID ooid = nnnoa.GetNnoi().GetOid();
						// If Meta representation have an id == null, then
						// this is a new object
						// it must be inserted, else just update
						// reference
						if (ooid == null)
						{
							ooid = InsertNonNativeObject(null, nnnoa.GetNnoi(), true);
						}
						UpdateObjectReference(nnnoa.GetUpdatePosition(), ooid, writeInTransaction);
						nbAppliedChanges++;
					}
				}
				NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.SetAttributeToNullAction satna = null;
				// For attribute that have been set to null
				for (int i = 0; i < objectComparator.GetAttributeToSetToNull().Count; i++)
				{
					satna = (NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.SetAttributeToNullAction)objectComparator
						.GetAttributeToSetToNull()[i];
					UpdateObjectReference(satna.GetUpdatePosition(), NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
						.NullObjectId, writeInTransaction);
					nbAppliedChanges++;
				}
				NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.ArrayModifyElement ame = null;
				// For attribute that have been set to null
				for (int i = 0; i < objectComparator.GetArrayChanges().Count; i++)
				{
					ame = (NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.ArrayModifyElement)objectComparator
						.GetArrayChanges()[i];
					if (!ame.SupportInPlaceUpdate())
					{
						break;
					}
					fsi.SetReadPosition(ame.GetArrayPositionDefinition());
					long arrayPosition = fsi.ReadLong();
					// If we reach this line,the ArrayModifyElement
					// suuports In Place Update so it must be a Native
					// Object Info!
					// The cast is safe :-)
					UpdateArrayElement(arrayPosition, ame.GetArrayElementIndexToChange(), (NeoDatis.Odb.Core.Layers.Layer2.Meta.NativeObjectInfo
						)ame.GetNewValue(), writeInTransaction);
					nbAppliedChanges++;
				}
			}
			// }// only direct attribute have been changed
			return nbAppliedChanges;
		}

		private bool CanDoInPlaceUpdate(long updatePosition, string value)
		{
			fsi.SetReadPosition(updatePosition + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.NativeObjectOffsetDataArea);
			int totalSize = fsi.ReadInt("String total size");
			int stringNumberOfBytes = byteArrayConverter.GetNumberOfBytesOfAString(value, true
				);
			// Checks if there is enough space to store this new string in place
			return totalSize >= stringNumberOfBytes;
		}

		public virtual string DepthToSpaces()
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			for (int i = 0; i < currentDepth; i++)
			{
				buffer.Append("  ");
			}
			return buffer.ToString();
		}

		private void WriteBlockSizeAt(long writePosition, int blockSize, bool writeInTransaction
			, object @object)
		{
			if (blockSize < 0)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.NegativeBlockSize
					.AddParameter(writePosition).AddParameter(blockSize).AddParameter(@object.ToString
					()));
			}
			long currentPosition = fsi.GetPosition();
			fsi.SetWritePosition(writePosition, writeInTransaction);
			fsi.WriteInt(blockSize, writeInTransaction, "block size");
			// goes back where we were
			fsi.SetWritePosition(currentPosition, writeInTransaction);
		}

		/// <summary>
		/// TODO check if we should pass the position instead of requesting if to fsi
		/// <pre>
		/// Write a collection to the database
		/// This is done by writing the number of element s and then the position of all elements.
		/// </summary>
		/// <remarks>
		/// TODO check if we should pass the position instead of requesting if to fsi
		/// <pre>
		/// Write a collection to the database
		/// This is done by writing the number of element s and then the position of all elements.
		/// Example : a list with two string element : 'ola' and 'chico'
		/// write 2 (as an int) : the number of elements
		/// write two times 0 (as long) to reserve the space for the elements positions
		/// then write the string 'ola', and keeps its position in the 'positions' array of long
		/// then write the string 'chico' and keeps its position in the 'positions' array of long
		/// Then write back all the positions (in this case , 2 positions) after the size of the collection
		/// &lt;pre&gt;
		/// &#064;param coi
		/// &#064;param writeInTransaction
		/// &#064;
		/// </remarks>
		private long WriteCollection(NeoDatis.Odb.Core.Layers.Layer2.Meta.CollectionObjectInfo
			 coi, bool writeInTransaction)
		{
			long firstObjectPosition = 0;
			long[] attributeIdentifications;
			long startPosition = fsi.GetPosition();
			WriteNativeObjectHeader(coi.GetOdbTypeId(), coi.IsNull(), NeoDatis.Odb.Impl.Core.Layers.Layer3.Block.BlockTypes
				.BlockTypeCollectionObject, writeInTransaction);
			if (coi.IsNull())
			{
				return startPosition;
			}
			System.Collections.Generic.ICollection<NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
				> collection = coi.GetCollection();
			int collectionSize = collection.Count;
			System.Collections.IEnumerator iterator = collection.GetEnumerator();
			// write the real type of the collection
			fsi.WriteString(coi.GetRealCollectionClassName(), false, writeInTransaction);
			// write the size of the collection
			fsi.WriteInt(collectionSize, writeInTransaction, "collection size");
			// build a n array to store all element positions
			attributeIdentifications = new long[collectionSize];
			// Gets the current position, to know later where to put the
			// references
			firstObjectPosition = fsi.GetPosition();
			// reserve space for object positions : write 'collectionSize' long
			// with zero to store each object position
			for (int i = 0; i < collectionSize; i++)
			{
				fsi.WriteLong(0, writeInTransaction, "collection element pos ", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
					.DataWriteAction);
			}
			int currentElement = 0;
			NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo element = null;
			while (iterator.MoveNext())
			{
				element = (NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo)iterator.Current;
				attributeIdentifications[currentElement] = InternalStoreObjectWrapper(element);
				currentElement++;
			}
			long positionAfterWrite = fsi.GetPosition();
			// now that all objects have been stored, sets their position in the
			// space that have been reserved
			fsi.SetWritePosition(firstObjectPosition, writeInTransaction);
			for (int i = 0; i < collectionSize; i++)
			{
				fsi.WriteLong(attributeIdentifications[i], writeInTransaction, "collection element real pos "
					, NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction.DataWriteAction);
			}
			// Goes back to the end of the array
			fsi.SetWritePosition(positionAfterWrite, writeInTransaction);
			return startPosition;
		}

		/// <summary>
		/// <pre>
		/// Write an array to the database
		/// This is done by writing :
		/// - the array type : array
		/// - the array element type (String if it os a String [])
		/// - the position of the non native type, if element are non java / C# native
		/// - the number of element s and then the position of all elements.
		/// </summary>
		/// <remarks>
		/// <pre>
		/// Write an array to the database
		/// This is done by writing :
		/// - the array type : array
		/// - the array element type (String if it os a String [])
		/// - the position of the non native type, if element are non java / C# native
		/// - the number of element s and then the position of all elements.
		/// Example : an array with two string element : 'ola' and 'chico'
		/// write 22 : array
		/// write  20 : array of STRING
		/// write 0 : it is a java native object
		/// write 2 (as an int) : the number of elements
		/// write two times 0 (as long) to reserve the space for the elements positions
		/// then write the string 'ola', and keeps its position in the 'positions' array of long
		/// then write the string 'chico' and keeps its position in the 'positions' array of long
		/// Then write back all the positions (in this case , 2 positions) after the size of the array
		/// Example : an array with two User element : user1 and user2
		/// write 22 : array
		/// write  23 : array of NON NATIVE Objects
		/// write 251 : if 250 is the position of the user class info in database
		/// write 2 (as an int) : the number of elements
		/// write two times 0 (as long) to reserve the space for the elements positions
		/// then write the user user1, and keeps its position in the 'positions' array of long
		/// then write the user user2 and keeps its position in the 'positions' array of long
		/// &lt;pre&gt;
		/// &#064;param object
		/// &#064;param odbType
		/// &#064;param position
		/// &#064;param writeInTransaction
		/// &#064;
		/// </remarks>
		private long WriteArray(ArrayObjectInfo aoi, bool writeInTransaction)
		{
			long firstObjectPosition = 0;
			long[] attributeIdentifications;
			long startPosition = fsi.GetPosition();
			WriteNativeObjectHeader(aoi.GetOdbTypeId(), aoi.IsNull(), NeoDatis.Odb.Impl.Core.Layers.Layer3.Block.BlockTypes
				.BlockTypeArrayObject, writeInTransaction);
			if (aoi.IsNull())
			{
				return startPosition;
			}
			object[] array = aoi.GetArray();
			int arraySize = array.Length;
			// Writes the fact that it is an array
			fsi.WriteString(aoi.GetRealArrayComponentClassName(), false, writeInTransaction);
			// write the size of the array
			fsi.WriteInt(arraySize, writeInTransaction, "array size");
			// build a n array to store all element positions
			attributeIdentifications = new long[arraySize];
			// Gets the current position, to know later where to put the
			// references
			firstObjectPosition = fsi.GetPosition();
			// reserve space for object positions : write 'arraySize' long
			// with zero to store each object position
			for (int i = 0; i < arraySize; i++)
			{
				fsi.WriteLong(0, writeInTransaction, "array element pos ", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
					.DataWriteAction);
			}
			NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo element = null;
			for (int i = 0; i < arraySize; i++)
			{
				element = (NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo)array[i];
				if (element == null || element.IsNull())
				{
					// TODO Check this
					attributeIdentifications[i] = NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
						.NullObjectIdId;
					continue;
				}
				attributeIdentifications[i] = InternalStoreObjectWrapper(element);
			}
			long positionAfterWrite = fsi.GetPosition();
			// now that all objects have been stored, sets their position in the
			// space that have been reserved
			fsi.SetWritePosition(firstObjectPosition, writeInTransaction);
			for (int i = 0; i < arraySize; i++)
			{
				fsi.WriteLong(attributeIdentifications[i], writeInTransaction, "array real element pos"
					, NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction.DataWriteAction);
			}
			// Gos back to the end of the array
			fsi.SetWritePosition(positionAfterWrite, writeInTransaction);
			return startPosition;
		}

		/// <summary>
		/// <pre>
		/// Write a map to the database
		/// This is done by writing the number of element s and then the key and value pair of all elements.
		/// </summary>
		/// <remarks>
		/// <pre>
		/// Write a map to the database
		/// This is done by writing the number of element s and then the key and value pair of all elements.
		/// Example : a map with two string element : '1/olivier' and '2/chico'
		/// write 2 (as an int) : the number of elements
		/// write 4 times 0 (as long) to reserve the space for the elements positions
		/// then write the object '1' and 'olivier', and keeps the two posiitons in the 'positions' array of long
		/// then write the object '2' and the string chico' and keep the two position in the 'positions' array of long
		/// Then write back all the positions (in this case , 4 positions) after the size of the map
		/// &#064;param object
		/// &#064;param writeInTransaction To specify if these writes must be done in or out of a transaction
		/// &#064;
		/// </remarks>
		private long WriteMap(NeoDatis.Odb.Core.Layers.Layer2.Meta.MapObjectInfo moi, bool
			 writeInTransaction)
		{
			long firstObjectPosition = 0;
			long[] positions;
			long startPosition = fsi.GetPosition();
			WriteNativeObjectHeader(moi.GetOdbTypeId(), moi.IsNull(), NeoDatis.Odb.Impl.Core.Layers.Layer3.Block.BlockTypes
				.BlockTypeMapObject, writeInTransaction);
			if (moi.IsNull())
			{
				return startPosition;
			}
			System.Collections.Generic.IDictionary<NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
				, NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo> map = moi.GetMap();
			int mapSize = map.Count;
			System.Collections.Generic.IEnumerator<NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
				> keys = map.Keys.GetEnumerator();
			// write the map class
			fsi.WriteString(moi.GetRealMapClassName(), false, writeInTransaction);
			// write the size of the map
			fsi.WriteInt(mapSize, writeInTransaction, "map size");
			// build a n array to store all element positions
			positions = new long[mapSize * 2];
			// Gets the current position, to know later where to put the
			// references
			firstObjectPosition = fsi.GetPosition();
			// reserve space for object positions : write 'mapSize*2' long
			// with zero to store each object position
			for (int i = 0; i < mapSize * 2; i++)
			{
				fsi.WriteLong(0, writeInTransaction, "map element pos", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
					.DataWriteAction);
			}
			int currentElement = 0;
			while (keys.MoveNext())
			{
				NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo key = keys.Current;
				NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo value = map[key];
				NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType keyType = NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
					.GetFromClass(key.GetType());
				NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType valueType = NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
					.GetFromClass(value.GetType());
				positions[currentElement++] = InternalStoreObjectWrapper(key);
				positions[currentElement++] = InternalStoreObjectWrapper(value);
			}
			long positionAfterWrite = fsi.GetPosition();
			// now that all objects have been stored, sets their position in the
			// space that have been reserved
			fsi.SetWritePosition(firstObjectPosition, writeInTransaction);
			for (int i = 0; i < mapSize * 2; i++)
			{
				fsi.WriteLong(positions[i], writeInTransaction, "map real element pos", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
					.DataWriteAction);
			}
			// Gos back to the end of the array
			fsi.SetWritePosition(positionAfterWrite, writeInTransaction);
			return startPosition;
		}

		/// <summary>
		/// This method is used to store the object : natibe or non native and return
		/// a number : - The position of the object if it is a native object - The
		/// oid (as a negative number) if it is a non native object
		/// </summary>
		/// <param name="aoi"></param>
		/// <returns></returns>
		/// <exception cref="System.Exception">System.Exception</exception>
		private long InternalStoreObjectWrapper(NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
			 aoi)
		{
			if (aoi.IsNative())
			{
				return InternalStoreObject((NeoDatis.Odb.Core.Layers.Layer2.Meta.NativeObjectInfo
					)aoi);
			}
			if (aoi.IsNonNativeObject())
			{
				NeoDatis.Odb.OID oid = StoreObject(null, (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
					)aoi);
				return -oid.GetObjectId();
			}
			// Object references are references to object already stored.
			// But in the case of map, the reference can appear before the real
			// object (as order may change)
			// If objectReference.getOid() is null, it is the case. In this case,
			// We take the object being referenced and stores it directly.
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectReference objectReference = (NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectReference
				)aoi;
			if (objectReference.GetOid() == null)
			{
				NeoDatis.Odb.OID oid = StoreObject(null, objectReference.GetNnoi());
				return -oid.GetObjectId();
			}
			return -objectReference.GetOid().GetObjectId();
		}

		protected virtual void WriteNullNativeObjectHeader(int OdbTypeId, bool writeInTransaction
			)
		{
			WriteNativeObjectHeader(OdbTypeId, true, NeoDatis.Odb.Impl.Core.Layers.Layer3.Block.BlockTypes
				.BlockTypeNativeNullObject, writeInTransaction);
		}

		protected virtual void WriteNonNativeNullObjectHeader(NeoDatis.Odb.OID classInfoId
			, bool writeInTransaction)
		{
			// Block size
			fsi.WriteInt(NonNativeHeaderBlockSize, writeInTransaction, "block size");
			// Block type
			fsi.WriteByte(NeoDatis.Odb.Impl.Core.Layers.Layer3.Block.BlockTypes.BlockTypeNonNativeNullObject
				, writeInTransaction);
			// class info id
			fsi.WriteLong(classInfoId.GetObjectId(), writeInTransaction, "null non native obj class info position"
				, NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction.DataWriteAction);
		}

		/// <summary>Write the header of a native attribute</summary>
		/// <param name="odbTypeId"></param>
		/// <param name="isNull"></param>
		/// <param name="writeDataInTransaction"></param>
		/// <></>
		protected virtual void WriteNativeObjectHeader(int odbTypeId, bool isNull, byte blockType
			, bool writeDataInTransaction)
		{
			byte[] bytes = new byte[10];
			bytes[0] = NativeHeaderBlockSizeByte[0];
			bytes[1] = NativeHeaderBlockSizeByte[1];
			bytes[2] = NativeHeaderBlockSizeByte[2];
			bytes[3] = NativeHeaderBlockSizeByte[3];
			bytes[4] = blockType;
			byte[] bytesTypeId = byteArrayConverter.IntToByteArray(odbTypeId);
			bytes[5] = bytesTypeId[0];
			bytes[6] = bytesTypeId[1];
			bytes[7] = bytesTypeId[2];
			bytes[8] = bytesTypeId[3];
			bytes[9] = byteArrayConverter.BooleanToByteArray(isNull)[0];
			fsi.WriteBytes(bytes, writeDataInTransaction, "NativeObjectHeader");
		}

		/// <exception cref="Java.Lang.NumberFormatException"></exception>
		/// <exception cref="System.IO.IOException"></exception>
		public virtual long SafeOverWriteAtomicNativeObject(long position, NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo
			 newAnoi, bool writeInTransaction)
		{
			// If the attribute an a non fix ize, check if this write is safe
			if (NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.HasFixSize(newAnoi.GetOdbTypeId(
				)))
			{
				fsi.SetWritePosition(position, writeInTransaction);
				return WriteAtomicNativeObject(newAnoi, writeInTransaction);
			}
			if (NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.IsStringOrBigDicemalOrBigInteger
				(newAnoi.GetOdbTypeId()))
			{
				fsi.SetReadPosition(position + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
					.NativeObjectOffsetDataArea);
				int totalSize = fsi.ReadInt("String total size");
				int stringNumberOfBytes = byteArrayConverter.GetNumberOfBytesOfAString(newAnoi.GetObject
					().ToString(), true);
				// Checks if there is enough space to store this new string in place
				bool canUpdate = totalSize >= stringNumberOfBytes;
				if (canUpdate)
				{
					fsi.SetWritePosition(position, writeInTransaction);
					return WriteAtomicNativeObject(newAnoi, writeInTransaction, totalSize);
				}
			}
			return -1;
		}

		public virtual long WriteEnumNativeObject(NeoDatis.Odb.Core.Layers.Layer2.Meta.EnumNativeObjectInfo
			 anoi, bool writeInTransaction)
		{
			long startPosition = fsi.GetPosition();
			int odbTypeId = anoi.GetOdbTypeId();
			WriteNativeObjectHeader(odbTypeId, anoi.IsNull(), NeoDatis.Odb.Impl.Core.Layers.Layer3.Block.BlockTypes
				.BlockTypeNativeObject, writeInTransaction);
			// Writes the Enum ClassName
			fsi.WriteLong(anoi.GetEnumClassInfo().GetId().GetObjectId(), writeInTransaction, 
				"enum class info id", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction.DataWriteAction
				);
			// Write the Enum String value
			fsi.WriteString(anoi.GetObject().ToString(), writeInTransaction, true, -1);
			return startPosition;
		}

		/// <summary>Writes a natibve attribute</summary>
		/// <param name="anoi"></param>
		/// <param name="writeInTransaction">
		/// To specify if data must be written in the transaction or
		/// directly to database file
		/// </param>
		/// <returns>The object position</returns>
		/// <exception cref="Java.Lang.NumberFormatException">Java.Lang.NumberFormatException
		/// 	</exception>
		/// <>* TODO the block is set to 0</>
		public virtual long WriteAtomicNativeObject(NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo
			 anoi, bool writeInTransaction)
		{
			return WriteAtomicNativeObject(anoi, writeInTransaction, -1);
		}

		public virtual long WriteAtomicNativeObject(NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo
			 anoi, bool writeInTransaction, int totalSpaceIfString)
		{
			long startPosition = fsi.GetPosition();
			int odbTypeId = anoi.GetOdbTypeId();
			WriteNativeObjectHeader(odbTypeId, anoi.IsNull(), NeoDatis.Odb.Impl.Core.Layers.Layer3.Block.BlockTypes
				.BlockTypeNativeObject, writeInTransaction);
			if (anoi.IsNull())
			{
				// Even if object is null, reserve space for to simplify/enable in
				// place update
				fsi.EnsureSpaceFor(anoi.GetOdbType());
				return startPosition;
			}
			object @object = anoi.GetObject();
			switch (odbTypeId)
			{
				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.ByteId:
				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeByteId:
				{
					fsi.WriteByte(((byte)@object), writeInTransaction);
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.BooleanId:
				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeBooleanId:
				{
					fsi.WriteBoolean(((bool)@object), writeInTransaction);
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.CharacterId:
				{
					fsi.WriteChar(((char)@object), writeInTransaction);
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeCharId:
				{
					fsi.WriteChar(@object.ToString()[0], writeInTransaction);
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.FloatId:
				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeFloatId:
				{
					fsi.WriteFloat(((float)@object), writeInTransaction);
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.DoubleId:
				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeDoubleId:
				{
					fsi.WriteDouble(((double)@object), writeInTransaction);
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.IntegerId:
				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeIntId:
				{
					fsi.WriteInt(((int)@object), writeInTransaction, "native attr");
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.LongId:
				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeLongId:
				{
					fsi.WriteLong(((long)@object), writeInTransaction, "native attr", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
						.DataWriteAction);
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.ShortId:
				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeShortId:
				{
					fsi.WriteShort(((short)@object), writeInTransaction);
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.BigDecimalId:
				{
					fsi.WriteBigDecimal((System.Decimal)@object, writeInTransaction);
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.BigIntegerId:
				{
					fsi.WriteBigInteger((System.Decimal)@object, writeInTransaction);
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.DateId:
				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.DateSqlId:
				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.DateTimestampId:
				{
					fsi.WriteDate((System.DateTime)@object, writeInTransaction);
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.StringId:
				{
					fsi.WriteString((string)@object, writeInTransaction, true, totalSpaceIfString);
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.OidId:
				{
					long oid = ((NeoDatis.Odb.Impl.Core.Oid.OdbObjectOID)@object).GetObjectId();
					fsi.WriteLong(oid, writeInTransaction, "ODB OID", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
						.DataWriteAction);
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.ObjectOidId:
				{
					long ooid = ((NeoDatis.Odb.Impl.Core.Oid.OdbObjectOID)@object).GetObjectId();
					fsi.WriteLong(ooid, writeInTransaction, "ODB OID", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
						.DataWriteAction);
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.ClassOidId:
				{
					long coid = ((NeoDatis.Odb.Impl.Core.Oid.OdbClassOID)@object).GetObjectId();
					fsi.WriteLong(coid, writeInTransaction, "ODB OID", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
						.DataWriteAction);
					break;
				}

				default:
				{
					// FIXME replace RuntimeException by a
					throw new System.Exception("native type with odb type id " + odbTypeId + " (" + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
						.GetNameFromId(odbTypeId) + ") for attribute ? is not suported");
					break;
				}
			}
			return startPosition;
		}

		/// <summary>
		/// Updates the previous object position field of the object at
		/// objectPosition
		/// </summary>
		/// <param name="objectOID"></param>
		/// <param name="previousObjectOID"></param>
		/// <param name="writeInTransaction"></param>
		/// <></>
		public virtual void UpdatePreviousObjectFieldOfObjectInfo(NeoDatis.Odb.OID objectOID
			, NeoDatis.Odb.OID previousObjectOID, bool writeInTransaction)
		{
			long objectPosition = idManager.GetObjectPositionWithOid(objectOID, true);
			fsi.SetWritePosition(objectPosition + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.ObjectOffsetPreviousObjectOid, writeInTransaction);
			WriteOid(previousObjectOID, writeInTransaction, "prev object position", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
				.PointerWriteAction);
		}

		/// <summary>Update next object oid field of the object at the specific position</summary>
		/// <param name="objectOID"></param>
		/// <param name="nextObjectOID"></param>
		/// <param name="writeInTransaction"></param>
		/// <></>
		public virtual void UpdateNextObjectFieldOfObjectInfo(NeoDatis.Odb.OID objectOID, 
			NeoDatis.Odb.OID nextObjectOID, bool writeInTransaction)
		{
			long objectPosition = idManager.GetObjectPositionWithOid(objectOID, true);
			fsi.SetWritePosition(objectPosition + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.ObjectOffsetNextObjectOid, writeInTransaction);
			WriteOid(nextObjectOID, writeInTransaction, "next object oid of object info", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
				.PointerWriteAction);
		}

		/// <summary>Mark a block as deleted</summary>
		/// <returns>The block size</returns>
		/// <param name="currentPosition"></param>
		/// <></>
		public virtual int MarkAsDeleted(long currentPosition, NeoDatis.Odb.OID oid, bool
			 writeInTransaction)
		{
			fsi.SetReadPosition(currentPosition);
			int blockSize = fsi.ReadInt();
			fsi.SetWritePosition(currentPosition + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.NativeObjectOffsetBlockType, writeInTransaction);
			// Do not write block size, leave it as it is, to know the available
			// space for future use
			fsi.WriteByte(NeoDatis.Odb.Impl.Core.Layers.Layer3.Block.BlockTypes.BlockTypeDeleted
				, writeInTransaction);
			StoreFreeSpace(currentPosition, blockSize);
			return blockSize;
		}

		public virtual void StoreFreeSpace(long currentPosition, int blockSize)
		{
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
			{
				NeoDatis.Tool.DLogger.Debug("Storing free space at position " + currentPosition +
					 " | block size = " + blockSize);
			}
		}

		/// <summary>Writes a pointer block : A pointer block is like a goto.</summary>
		/// <remarks>
		/// Writes a pointer block : A pointer block is like a goto. It can be used
		/// for example when an instance has been updated. To enable all the
		/// references to it to be updated, we just create o pointer at the place of
		/// the updated instance. When searching for the instance, if the block type
		/// is POINTER, then the position will be set to the pointer position
		/// </remarks>
		/// <param name="currentPosition"></param>
		/// <param name="newObjectPosition"></param>
		/// <></>
		protected virtual void MarkAsAPointerTo(NeoDatis.Odb.OID oid, long currentPosition
			, long newObjectPosition)
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.FoundPointer
				.AddParameter(oid.GetObjectId()).AddParameter(newObjectPosition));
		}

		/// <summary>
		/// Updates the instance related field of the class info into the database
		/// file Updates the number of objects, the first object oid and the next
		/// class oid
		/// </summary>
		/// <param name="classInfo">The class info to be updated</param>
		/// <param name="writeInTransaction">To specify if it must be part of a transaction @
		/// 	</param>
		public virtual void UpdateInstanceFieldsOfClassInfo(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			 classInfo, bool writeInTransaction)
		{
			long currentPosition = fsi.GetPosition();
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogIdDebug))
			{
				NeoDatis.Tool.DLogger.Debug(DepthToSpaces() + "Start of updateInstanceFieldsOfClassInfo for "
					 + classInfo.GetFullClassName());
			}
			long position = classInfo.GetPosition() + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.ClassOffsetClassNbObjects;
			fsi.SetWritePosition(position, writeInTransaction);
			long nbObjects = classInfo.GetNumberOfObjects();
			fsi.WriteLong(nbObjects, writeInTransaction, "class info update nb objects", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
				.PointerWriteAction);
			WriteOid(classInfo.GetCommitedZoneInfo().first, writeInTransaction, "class info update first obj oid"
				, NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction.PointerWriteAction);
			WriteOid(classInfo.GetCommitedZoneInfo().last, writeInTransaction, "class info update last obj oid"
				, NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction.PointerWriteAction);
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogIdDebug))
			{
				NeoDatis.Tool.DLogger.Debug(DepthToSpaces() + "End of updateInstanceFieldsOfClassInfo for "
					 + classInfo.GetFullClassName());
			}
			fsi.SetWritePosition(currentPosition, writeInTransaction);
		}

		/// <summary>Updates the last instance field of the class info into the database file
		/// 	</summary>
		/// <param name="classInfoPosition">The class info to be updated</param>
		/// <param name="lastInstancePosition">The last instance position @</param>
		protected virtual void UpdateLastInstanceFieldOfClassInfoWithId(NeoDatis.Odb.OID 
			classInfoId, long lastInstancePosition)
		{
			long currentPosition = fsi.GetPosition();
			// TODO CHECK LOGIC of getting position of class using this method for
			// object)
			long classInfoPosition = idManager.GetObjectPositionWithOid(classInfoId, true);
			fsi.SetWritePosition(classInfoPosition + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.ClassOffsetClassLastObjectPosition, true);
			fsi.WriteLong(lastInstancePosition, true, "class info update last instance field"
				, NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction.PointerWriteAction);
			// TODO check if we need this
			fsi.SetWritePosition(currentPosition, true);
		}

		/// <summary>Updates the first instance field of the class info into the database file
		/// 	</summary>
		/// <param name="classInfoPosition">The class info to be updated</param>
		/// <param name="firstInstancePosition">The first instance position @</param>
		protected virtual void UpdateFirstInstanceFieldOfClassInfoWithId(NeoDatis.Odb.OID
			 classInfoId, long firstInstancePosition)
		{
			long currentPosition = fsi.GetPosition();
			// TODO CHECK LOGIC of getting position of class using this method for
			// object)
			long classInfoPosition = idManager.GetObjectPositionWithOid(classInfoId, true);
			fsi.SetWritePosition(classInfoPosition + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.ClassOffsetClassFirstObjectPosition, true);
			fsi.WriteLong(firstInstancePosition, true, "class info update first instance field"
				, NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction.PointerWriteAction);
			// TODO check if we need this
			fsi.SetWritePosition(currentPosition, true);
		}

		/// <summary>Updates the number of objects of the class info into the database file</summary>
		/// <param name="classInfoPosition">The class info to be updated</param>
		/// <param name="nbObjects">The number of object @</param>
		protected virtual void UpdateNbObjectsFieldOfClassInfo(NeoDatis.Odb.OID classInfoId
			, long nbObjects)
		{
			long currentPosition = fsi.GetPosition();
			long classInfoPosition = GetSession().GetMetaModel().GetClassInfoFromId(classInfoId
				).GetPosition();
			fsi.SetWritePosition(classInfoPosition + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.ClassOffsetClassNbObjects, true);
			fsi.WriteLong(nbObjects, true, "class info update nb objects", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
				.PointerWriteAction);
			// TODO check if we need this
			fsi.SetWritePosition(currentPosition, true);
		}

		/// <summary>
		/// <pre>
		/// Class User{
		/// private String name;
		/// private Function function;
		/// }
		/// When an object of type User is stored, it stores a reference to its function object.
		/// </summary>
		/// <remarks>
		/// <pre>
		/// Class User{
		/// private String name;
		/// private Function function;
		/// }
		/// When an object of type User is stored, it stores a reference to its function object.
		/// If the function is set to another, the pointer to the function object must be changed.
		/// for example, it was pointing to a function at the position 1407, the 1407 value is stored while
		/// writing the USer object, let's say at the position 528. To make the user point to another function object (which exist at the position 1890)
		/// The position 528 must be updated to 1890.
		/// </pre>
		/// </remarks>
		/// <param name="positionWhereTheReferenceIsStored"></param>
		/// <param name="newOid"></param>
		/// <></>
		public virtual void UpdateObjectReference(long positionWhereTheReferenceIsStored, 
			NeoDatis.Odb.OID newOid, bool writeInTransaction)
		{
			long position = positionWhereTheReferenceIsStored;
			if (position < 0)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.NegativePosition
					.AddParameter(position));
			}
			fsi.SetWritePosition(position, writeInTransaction);
			// Ids are always stored as negative value to differ from a position!
			long oid = NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant.NullObjectIdId;
			if (newOid != null)
			{
				oid = -newOid.GetObjectId();
			}
			fsi.WriteLong(oid, writeInTransaction, "object reference", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
				.PointerWriteAction);
		}

		/// <summary>
		/// In place update for array element, only do in place update for atomic
		/// native fixed size elements
		/// </summary>
		/// <param name="arrayPosition"></param>
		/// <param name="arrayElementIndexToChange"></param>
		/// <param name="newValue"></param>
		/// <returns>true if in place update has been done,false if not</returns>
		/// <exception cref="System.Exception">System.Exception</exception>
		private bool UpdateArrayElement(long arrayPosition, int arrayElementIndexToChange
			, NeoDatis.Odb.Core.Layers.Layer2.Meta.NativeObjectInfo newValue, bool writeInTransaction
			)
		{
			// block size, block type, odb typeid,is null?
			long offset = NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Integer.GetSize() + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
				.Byte.GetSize() + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Integer.GetSize()
				 + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Boolean.GetSize();
			fsi.SetReadPosition(arrayPosition + offset);
			// read class name of array elements
			string arrayElementClassName = fsi.ReadString(false);
			// TODO try to get array element type from the ArrayObjectInfo
			// Check if the class has fixed size : array support in place update
			// only for fixed size class like int, long, date,...
			// String array,for example do not support in place update
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType arrayElementType = NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
				.GetFromName(arrayElementClassName);
			if (!arrayElementType.IsAtomicNative() || !arrayElementType.HasFixSize())
			{
				return false;
			}
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ArrayObjectInfo a = null;
			// reads the size of the array
			int arraySize = fsi.ReadInt();
			if (arrayElementIndexToChange >= arraySize)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.InplaceUpdateNotPossibleForArray
					.AddParameter(arraySize).AddParameter(arrayElementIndexToChange));
			}
			// Gets the position where to write the object
			// Skip the positions where we have the pointers to each array element
			// then
			// jump to the right position
			long skip = arrayElementIndexToChange * NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
				.Long.GetSize();
			fsi.SetReadPosition(fsi.GetPosition() + skip);
			long elementArrayPosition = fsi.ReadLong();
			fsi.SetWritePosition(elementArrayPosition, writeInTransaction);
			// Actually update the array element
			WriteNativeObjectInfo(newValue, elementArrayPosition, true, writeInTransaction);
			return true;
		}

		public virtual void Flush()
		{
			fsi.Flush();
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer3.IIdManager GetIdManager()
		{
			return idManager;
		}

		public virtual void Close()
		{
			objectReader = null;
			if (idManager != null)
			{
				idManager.Clear();
				idManager = null;
			}
			storageEngine = null;
			fsi.Close();
			fsi = null;
		}

		public static int GetNbInPlaceUpdates()
		{
			return nbInPlaceUpdates;
		}

		public static void SetNbInPlaceUpdates(int nbInPlaceUpdates)
		{
			NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.AbstractObjectWriter.nbInPlaceUpdates
				 = nbInPlaceUpdates;
		}

		public static int GetNbNormalUpdates()
		{
			return nbNormalUpdates;
		}

		public static void SetNbNormalUpdates(int nbNormalUpdates)
		{
			NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.AbstractObjectWriter.nbNormalUpdates 
				= nbNormalUpdates;
		}

		public static void ResetNbUpdates()
		{
			nbInPlaceUpdates = 0;
			nbNormalUpdates = 0;
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer3.Engine.IFileSystemInterface GetFsi
			()
		{
			return fsi;
		}

		public virtual NeoDatis.Odb.OID Delete(NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader
			 header)
		{
			NeoDatis.Odb.Core.Transaction.ISession lsession = GetSession();
			NeoDatis.Odb.Core.Transaction.ICache cache = lsession.GetCache();
			long objectPosition = header.GetPosition();
			NeoDatis.Odb.OID classInfoId = header.GetClassInfoId();
			NeoDatis.Odb.OID oid = header.GetOid();
			// gets class info from in memory meta model
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci = GetSession().GetMetaModel().GetClassInfoFromId
				(classInfoId);
			bool withIndex = !ci.GetIndexes().IsEmpty();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoi = null;
			// When there is index,we must *always* load the old meta representation
			// to compute index keys
			if (withIndex)
			{
				nnoi = objectReader.ReadNonNativeObjectInfoFromPosition(ci, header.GetOid(), objectPosition
					, true, false);
			}
			// a boolean value to indicate if object is in connected zone or not
			// This will be used to know if work can be done out of transaction
			// for unconnected object,changes can be written directly, else we must
			// use Transaction (using WriteAction)
			bool objectIsInConnectedZone = cache.ObjectWithIdIsInCommitedZone(header.GetOid()
				);
			// triggers
			// FIXME
			triggerManager.ManageDeleteTriggerBefore(ci.GetFullClassName(), null, header.GetOid
				());
			long nbObjects = ci.GetNumberOfObjects();
			NeoDatis.Odb.OID previousObjectOID = header.GetPreviousObjectOID();
			NeoDatis.Odb.OID nextObjectOID = header.GetNextObjectOID();
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
			{
				NeoDatis.Tool.DLogger.Debug("Deleting object with id " + header.GetOid() + " - In connected zone ="
					 + objectIsInConnectedZone + " -  with index =" + withIndex);
				NeoDatis.Tool.DLogger.Debug("position =  " + objectPosition + " | prev oid = " + 
					previousObjectOID + " | next oid = " + nextObjectOID);
			}
			bool isFirstObject = previousObjectOID == null;
			bool isLastObject = nextObjectOID == null;
			bool mustUpdatePreviousObjectPointers = false;
			bool mustUpdateNextObjectPointers = false;
			bool mustUpdateLastObjectOfCI = false;
			if (isFirstObject || isLastObject)
			{
				if (isFirstObject)
				{
					// The deleted object is the first, must update first instance
					// OID field of the class
					if (objectIsInConnectedZone)
					{
						// update first object oid of the class info in memory
						ci.GetCommitedZoneInfo().first = nextObjectOID;
					}
					else
					{
						// update first object oid of the class info in memory
						ci.GetUncommittedZoneInfo().first = nextObjectOID;
					}
					if (nextObjectOID != null)
					{
						// Update next object 'previous object oid' to null
						UpdatePreviousObjectFieldOfObjectInfo(nextObjectOID, null, objectIsInConnectedZone
							);
						mustUpdateNextObjectPointers = true;
					}
				}
				// It can be first and last
				if (isLastObject)
				{
					// The deleted object is the last, must update last instance
					// OID field of the class
					// update last object position of the class info in memory
					if (objectIsInConnectedZone)
					{
						// the object is a committed object
						ci.GetCommitedZoneInfo().last = previousObjectOID;
					}
					else
					{
						// The object is not committed and it is the last and is
						// being deleted
						ci.GetUncommittedZoneInfo().last = previousObjectOID;
					}
					if (previousObjectOID != null)
					{
						// Update 'next object oid' of previous object to null
						// if we are in unconnected zone, change can be done
						// directly,else it must be done in transaction
						UpdateNextObjectFieldOfObjectInfo(previousObjectOID, null, objectIsInConnectedZone
							);
						// Now update data of the cache
						mustUpdatePreviousObjectPointers = true;
						mustUpdateLastObjectOfCI = true;
					}
				}
			}
			else
			{
				// Normal case, the deleted object has previous and next object
				// pull the deleted object
				// Mark the 'next object oid field' of the previous object
				// pointing the next object
				UpdateNextObjectFieldOfObjectInfo(previousObjectOID, nextObjectOID, objectIsInConnectedZone
					);
				// Mark the 'previous object position field' of the next object
				// pointing the previous object
				UpdatePreviousObjectFieldOfObjectInfo(nextObjectOID, previousObjectOID, objectIsInConnectedZone
					);
				mustUpdateNextObjectPointers = true;
				mustUpdatePreviousObjectPointers = true;
			}
			if (mustUpdateNextObjectPointers)
			{
				UpdateNextObjectPreviousPointersInCache(nextObjectOID, previousObjectOID, cache);
			}
			if (mustUpdatePreviousObjectPointers)
			{
				NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader oih = UpdatePreviousObjectNextPointersInCache
					(nextObjectOID, previousObjectOID, cache);
				if (mustUpdateLastObjectOfCI)
				{
					ci.SetLastObjectInfoHeader(oih);
				}
			}
			NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel metaModel = lsession.GetMetaModel(
				);
			// Saves the fact that something has changed in the class (number of
			// objects and/or last object oid)
			metaModel.AddChangedClass(ci);
			if (objectIsInConnectedZone)
			{
				ci.GetCommitedZoneInfo().DecreaseNbObjects();
			}
			else
			{
				ci.GetUncommittedZoneInfo().DecreaseNbObjects();
			}
			// Manage deleting the last object of the committed zone
			NeoDatis.Odb.Core.Layers.Layer2.Meta.CIZoneInfo commitedZI = ci.GetCommitedZoneInfo
				();
			bool isLastObjectOfCommitedZone = oid.Equals(commitedZI.last);
			if (isLastObjectOfCommitedZone)
			{
				// Load the object info header of the last committed object
				NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader oih = objectReader.ReadObjectInfoHeaderFromOid
					(oid, true);
				// Updates last committed object id of the committed zone.
				// Here, it can be null, but there is no problem
				commitedZI.last = oih.GetPreviousObjectOID();
				// A simple check, if commitedZI.last is null, nbObject must be 0
				if (commitedZI.last == null && commitedZI.HasObjects())
				{
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.InternalError
						.AddParameter("The last object of the commited zone has been deleted but the Zone still have objects : nbobjects="
						 + commitedZI.GetNbObjects()));
				}
			}
			// Manage deleting the first object of the uncommitted zone
			NeoDatis.Odb.Core.Layers.Layer2.Meta.CIZoneInfo uncommitedZI = ci.GetUncommittedZoneInfo
				();
			bool isFirstObjectOfUncommitedZone = oid.Equals(uncommitedZI.first);
			if (isFirstObjectOfUncommitedZone)
			{
				if (uncommitedZI.HasObjects())
				{
					// Load the object info header of the first uncommitted object
					NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader oih = objectReader.ReadObjectInfoHeaderFromOid
						(oid, true);
					// Updates first uncommitted oid with the second uncommitted oid
					// Here, it can be null, but there is no problem
					uncommitedZI.first = oih.GetNextObjectOID();
				}
				else
				{
					uncommitedZI.first = null;
				}
			}
			if (isFirstObject && isLastObject)
			{
				// The object was the first and the last object => it was the only
				// object
				// There is no more objects of this type => must set to null the
				// ClassInfo LastObjectOID
				ci.SetLastObjectInfoHeader(null);
			}
			GetIdManager().UpdateIdStatus(header.GetOid(), NeoDatis.Odb.Impl.Core.Layers.Layer3.Oid.IDStatus
				.Deleted);
			// The update of the place must be done in transaction if object is in
			// committed zone, else it can be done directly in the file
			MarkAsDeleted(objectPosition, header.GetOid(), objectIsInConnectedZone);
			cache.MarkIdAsDeleted(header.GetOid());
			if (withIndex)
			{
				ManageIndexesForDelete(header.GetOid(), nnoi);
			}
			// triggers
			triggerManager.ManageDeleteTriggerAfter(ci.GetFullClassName(), null, header.GetOid
				());
			return header.GetOid();
		}

		public virtual void SetTriggerManager(NeoDatis.Odb.Core.Trigger.ITriggerManager triggerManager
			)
		{
			this.triggerManager = triggerManager;
		}
	}
}
