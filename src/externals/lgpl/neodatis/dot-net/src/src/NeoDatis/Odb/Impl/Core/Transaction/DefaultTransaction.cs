namespace NeoDatis.Odb.Impl.Core.Transaction
{
	/// <summary>
	/// <pre>
	/// The transaction class is used to guarantee ACID behavior.
	/// </summary>
	/// <remarks>
	/// <pre>
	/// The transaction class is used to guarantee ACID behavior. It keep tracks of all session
	/// operations. It uses the WriteAction class to store all changes that can not be written to the file
	/// before the commit.
	/// The transaction is held by The Session class and manage commits and rollbacks.
	/// All WriteActions are written in a transaction file to be sure to be able to commit and in case
	/// of very big transaction where all WriteActions can not be stored in memory.
	/// </pre>
	/// </remarks>
	/// <author>osmadja</author>
	public class DefaultTransaction : NeoDatis.Odb.Core.Transaction.ITransaction
	{
		/// <summary>the log module name</summary>
		public static readonly string LogId = "Transaction";

		/// <summary>To indicate if transaction was confirmed = committed</summary>
		private bool isCommited;

		/// <summary>The transaction creation time</summary>
		private long creationDateTime;

		/// <summary>
		/// All the pending writing that must be applied to actually commit the
		/// transaction
		/// </summary>
		private NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Transaction.IWriteAction
			> writeActions;

		/// <summary>The same write action is reused for successive writes</summary>
		public NeoDatis.Odb.Core.Transaction.IWriteAction currentWriteAction;

		/// <summary>The position of the next write for WriteAction</summary>
		public long currentWritePositionInWA;

		/// <summary>
		/// To indicate if all write actions are in memory - if not, transaction must
		/// read them from transaction file o commit the transaction
		/// </summary>
		private bool hasAllWriteActionsInMemory;

		/// <summary>The number of write actions</summary>
		public int numberOfWriteActions;

		/// <summary>A file interface to the transaction file - used to read/write the file</summary>
		public NeoDatis.Odb.Core.Layers.Layer3.Engine.IFileSystemInterface fsi;

		/// <summary>A file interface to the engine main file</summary>
		private NeoDatis.Odb.Core.Layers.Layer3.Engine.IFileSystemInterface fsiToApplyWriteActions;

		/// <summary>To indicate if transaction has already been persisted in file</summary>
		private bool hasBeenPersisted;

		/// <summary>
		/// When this flag is set,the transaction will not be deleted, but will be
		/// flagged as executed
		/// </summary>
		private bool archiveLog;

		/// <summary>To indicate if transaction was rollbacked</summary>
		private bool wasRollbacked;

		/// <summary>A name to set the transaction file name.</summary>
		/// <remarks>
		/// A name to set the transaction file name. Used when reading transaction
		/// file
		/// </remarks>
		private string overrideTransactionName;

		/// <summary>To indicate if transaction is read only</summary>
		private bool readOnlyMode;

		/// <summary>The transaction session</summary>
		public NeoDatis.Odb.Core.Transaction.ISession session;

		/// <summary>To indicate is transaction is used for local or remote engine</summary>
		private bool isLocal;

		private NeoDatis.Odb.Core.ICoreProvider provider;

		/// <summary>The main constructor</summary>
		/// <param name="session">The transaction session</param>
		/// <exception cref="System.IO.IOException">System.IO.IOException</exception>
		public DefaultTransaction(NeoDatis.Odb.Core.Transaction.ISession session)
		{
			Init(session);
		}

		/// <exception cref="System.IO.IOException"></exception>
		public DefaultTransaction(NeoDatis.Odb.Core.Transaction.ISession session, string 
			overrideTransactionName)
		{
			this.overrideTransactionName = overrideTransactionName;
			Init(session);
			this.readOnlyMode = true;
		}

		public DefaultTransaction(NeoDatis.Odb.Core.Transaction.ISession session, NeoDatis.Odb.Core.Layers.Layer3.Engine.IFileSystemInterface
			 fsiToApplyTransaction)
		{
			this.fsiToApplyWriteActions = fsiToApplyTransaction;
			Init(session);
			readOnlyMode = false;
		}

		public virtual void Init(NeoDatis.Odb.Core.Transaction.ISession session)
		{
			this.provider = NeoDatis.Odb.OdbConfiguration.GetCoreProvider();
			this.session = session;
			this.isCommited = false;
			creationDateTime = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			writeActions = new NeoDatis.Tool.Wrappers.List.OdbArrayList<NeoDatis.Odb.Core.Transaction.IWriteAction
				>(1000);
			hasAllWriteActionsInMemory = true;
			numberOfWriteActions = 0;
			hasBeenPersisted = false;
			wasRollbacked = false;
			currentWritePositionInWA = -1;
		}

		public virtual void Clear()
		{
			if (writeActions != null)
			{
				writeActions.Clear();
				writeActions = null;
			}
		}

		/// <summary>Reset the transaction</summary>
		public virtual void Reset()
		{
			Clear();
			Init(session);
			fsi = null;
		}

		/// <summary>Adds a write action to the transaction</summary>
		/// <param name="writeAction">The write action to be added</param>
		public virtual void AddWriteAction(NeoDatis.Odb.Core.Transaction.IWriteAction writeAction
			)
		{
			AddWriteAction(writeAction, true);
		}

		/// <summary>Adds a write action to the transaction</summary>
		/// <param name="writeAction">The write action to be added</param>
		/// <param name="persistWriteAcion">To indicate if write action must be persisted</param>
		public virtual void AddWriteAction(NeoDatis.Odb.Core.Transaction.IWriteAction writeAction
			, bool persistWriteAcion)
		{
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
			{
				NeoDatis.Tool.DLogger.Info("Adding WA in Transaction of session " + session.GetId
					());
			}
			if (writeAction.IsEmpty())
			{
				return;
			}
			CheckRollback();
			if (!hasBeenPersisted && persistWriteAcion)
			{
				Persist();
			}
			if (persistWriteAcion)
			{
				writeAction.Persist(fsi, numberOfWriteActions + 1);
			}
			// Only adds the writeaction to the list if the transaction keeps all in
			// memory
			if (hasAllWriteActionsInMemory)
			{
				writeActions.Add(writeAction);
			}
			numberOfWriteActions++;
			if (hasAllWriteActionsInMemory && numberOfWriteActions > NeoDatis.Odb.OdbConfiguration
				.GetMaxNumberOfWriteObjectPerTransaction())
			{
				hasAllWriteActionsInMemory = false;
				System.Collections.IEnumerator iterator = writeActions.GetEnumerator();
				NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction wa = null;
				while (iterator.MoveNext())
				{
					wa = (NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction)iterator.Current;
					wa.Clear();
				}
				writeActions.Clear();
				if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
				{
					NeoDatis.Tool.DLogger.Info("Number of objects has exceeded the max number " + numberOfWriteActions
						 + "/" + NeoDatis.Odb.OdbConfiguration.GetMaxNumberOfWriteObjectPerTransaction()
						 + ": switching to persistent transaction managment");
				}
			}
		}

		public virtual string GetName()
		{
			NeoDatis.Odb.Core.Layers.Layer3.IBaseIdentification p = fsiToApplyWriteActions.GetParameters
				();
			if (p is NeoDatis.Odb.Core.Layers.Layer3.IOFileParameter)
			{
				NeoDatis.Odb.Core.Layers.Layer3.IOFileParameter ifp = (NeoDatis.Odb.Core.Layers.Layer3.IOFileParameter
					)fsiToApplyWriteActions.GetParameters();
				System.Text.StringBuilder buffer = new System.Text.StringBuilder(ifp.GetCleanFileName
					()).Append("-").Append(creationDateTime).Append("-").Append(session.GetId()).Append
					(".transaction");
				return buffer.ToString();
			}
			if (p is NeoDatis.Odb.Core.Layers.Layer3.IOSocketParameter)
			{
				NeoDatis.Odb.Core.Layers.Layer3.IOSocketParameter sp = (NeoDatis.Odb.Core.Layers.Layer3.IOSocketParameter
					)fsiToApplyWriteActions.GetParameters();
				return sp.GetBaseIdentifier();
			}
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.UnsupportedIoType
				.AddParameter(p.GetType().FullName));
		}

		internal virtual NeoDatis.Odb.Core.Layers.Layer3.IBaseIdentification GetParameters
			(bool canWrite)
		{
			NeoDatis.Odb.Core.Layers.Layer3.IBaseIdentification p = fsiToApplyWriteActions.GetParameters
				();
			if (p is NeoDatis.Odb.Core.Layers.Layer3.IOFileParameter)
			{
				NeoDatis.Odb.Core.Layers.Layer3.IOFileParameter ifp = (NeoDatis.Odb.Core.Layers.Layer3.IOFileParameter
					)fsiToApplyWriteActions.GetParameters();
				System.Text.StringBuilder buffer = new System.Text.StringBuilder(ifp.GetDirectory
					()).Append("/").Append(ifp.GetCleanFileName()).Append("-").Append(creationDateTime
					).Append("-").Append(session.GetId()).Append(".transaction");
				return new NeoDatis.Odb.Core.Layers.Layer3.IOFileParameter(buffer.ToString(), canWrite
					, ifp.GetUserName(), ifp.GetPassword());
			}
			if (p is NeoDatis.Odb.Core.Layers.Layer3.IOSocketParameter)
			{
				NeoDatis.Odb.Core.Layers.Layer3.IOSocketParameter sp = (NeoDatis.Odb.Core.Layers.Layer3.IOSocketParameter
					)fsiToApplyWriteActions.GetParameters();
				return new NeoDatis.Odb.Core.Layers.Layer3.IOSocketParameter(sp.GetDestinationHost
					(), sp.GetPort(), sp.GetBaseIdentifier(), NeoDatis.Odb.Core.Layers.Layer3.IOSocketParameter
					.TypeTransaction, creationDateTime, null, null);
			}
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.UnsupportedIoType
				.AddParameter(p.GetType().FullName));
		}

		private void CheckFileAccess(bool canWrite)
		{
			CheckFileAccess(canWrite, null);
		}

		private void CheckFileAccess(bool canWrite, string fileName)
		{
			lock (this)
			{
				if (fsi == null)
				{
					NeoDatis.Odb.Core.Layers.Layer3.IBaseIdentification p = null;
					// to unable direct junit test of FileSystemInterface
					if (fsiToApplyWriteActions == null)
					{
						p = new NeoDatis.Odb.Core.Layers.Layer3.IOFileParameter(fileName, canWrite, null, 
							null);
					}
					else
					{
						p = GetParameters(canWrite);
					}
					// To enable unit test
					if (session != null)
					{
						isLocal = session.GetStorageEngine().IsLocal();
					}
					fsi = new NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.LocalFileSystemInterface("transaction"
						, session, p, false, NeoDatis.Odb.OdbConfiguration.GetDefaultBufferSizeForTransaction
						());
				}
			}
		}

		protected virtual void Persist()
		{
			CheckFileAccess(true);
			try
			{
				if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
				{
					NeoDatis.Tool.DLogger.Debug("# Persisting transaction " + GetName());
				}
				fsi.SetWritePosition(0, false);
				fsi.WriteBoolean(isCommited, false);
				fsi.WriteLong(creationDateTime, false, "creation date", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
					.DirectWriteAction);
				// Size
				fsi.WriteLong(0, false, "size", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
					.DirectWriteAction);
				hasBeenPersisted = true;
			}
			finally
			{
			}
		}

		public virtual NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Transaction.IWriteAction
			> GetWriteActions()
		{
			return writeActions;
		}

		public virtual long GetCreationDateTime()
		{
			return creationDateTime;
		}

		public virtual bool IsCommited()
		{
			return isCommited;
		}

		/// <summary>Mark te transaction file as committed</summary>
		/// <param name="isConfirmed"></param>
		private void SetCommited(bool isConfirmed)
		{
			this.isCommited = isConfirmed;
			CheckFileAccess(true);
			try
			{
				// TODO Check atomicity
				// Writes the number of write actions after the byte and date
				fsi.SetWritePositionNoVerification(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Byte
					.GetSize() + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Long.GetSize(), false);
				fsi.WriteLong(numberOfWriteActions, false, "nb write actions", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
					.DirectWriteAction);
				// FIXME The fsi.flush should not be called after the last write?
				fsi.Flush();
				// Only set useBuffer = false when it is a local database to avoid
				// net io overhead
				if (isLocal)
				{
					fsi.UseBuffer(false);
				}
				fsi.SetWritePositionNoVerification(0, false);
				fsi.WriteByte((byte)1, false);
			}
			finally
			{
			}
		}

		private void CheckRollback()
		{
			if (wasRollbacked)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OdbHasBeenRollbacked
					);
			}
		}

		public virtual void Rollback()
		{
			wasRollbacked = true;
			if (fsi != null)
			{
				fsi.Close();
				Delete();
			}
		}

		public virtual void Commit()
		{
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
			{
				NeoDatis.Tool.DLogger.Info("Commiting " + numberOfWriteActions + " write actions - In Memory : "
					 + hasAllWriteActionsInMemory + " - sid=" + session.GetId());
			}
			// Check if database has been rollbacked
			CheckRollback();
			// call the commit listeners
			ManageCommitListenersBefore();
			if (currentWriteAction != null && !currentWriteAction.IsEmpty())
			{
				AddWriteAction(currentWriteAction);
				currentWriteAction = null;
			}
			if (fsi == null && numberOfWriteActions != 0)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.TransactionAlreadyCommitedOrRollbacked
					);
			}
			if (numberOfWriteActions == 0 || readOnlyMode)
			{
				// FIXME call commitMetaModel in realOnlyMode?
				CommitMetaModel();
				// Nothing to do
				if (fsi != null)
				{
					fsi.Close();
					fsi = null;
				}
				if (session != null)
				{
					session.GetCache().ClearOnCommit();
				}
				return;
			}
			// Marks the transaction as committed
			SetCommited(true);
			// Apply the write actions the main database file
			ApplyTo();
			// Commit Meta Model changes
			CommitMetaModel();
			if (archiveLog)
			{
				fsi.SetWritePositionNoVerification(0, false);
				fsi.WriteByte((byte)2, false);
				fsi.GetIo().EnableAutomaticDelete(false);
				fsi.Close();
				fsi = null;
			}
			else
			{
				fsi.Close();
				Delete();
				fsi = null;
			}
			if (session != null)
			{
				session.GetCache().ClearOnCommit();
			}
			ManageCommitListenersAfter();
		}

		private void ManageCommitListenersAfter()
		{
			NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Layers.Layer3.ICommitListener
				> listeners = session.GetStorageEngine().GetCommitListeners();
			if (listeners == null || listeners.IsEmpty())
			{
				return;
			}
			System.Collections.Generic.IEnumerator<NeoDatis.Odb.Core.Layers.Layer3.ICommitListener
				> iterator = listeners.GetEnumerator();
			NeoDatis.Odb.Core.Layers.Layer3.ICommitListener commitListener = null;
			while (iterator.MoveNext())
			{
				commitListener = iterator.Current;
				commitListener.AfterCommit();
			}
		}

		private void ManageCommitListenersBefore()
		{
			NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Layers.Layer3.ICommitListener
				> listeners = session.GetStorageEngine().GetCommitListeners();
			if (listeners == null || listeners.IsEmpty())
			{
				return;
			}
			System.Collections.Generic.IEnumerator<NeoDatis.Odb.Core.Layers.Layer3.ICommitListener
				> iterator = listeners.GetEnumerator();
			NeoDatis.Odb.Core.Layers.Layer3.ICommitListener commitListener = null;
			while (iterator.MoveNext())
			{
				commitListener = iterator.Current;
				commitListener.BeforeCommit();
			}
		}

		/// <summary>
		/// Used to commit meta model : classes This is useful when running in client
		/// server mode TODO Check this
		/// </summary>
		protected virtual void CommitMetaModel()
		{
			NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel sessionMetaModel = session.GetMetaModel
				();
			// If meta model has not been modified, there is nothing to do
			if (!sessionMetaModel.HasChanged())
			{
				return;
			}
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
			{
				NeoDatis.Tool.DLogger.Debug("Start commitMetaModel");
			}
			NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel lastCommitedMetaModel = new NeoDatis.Odb.Core.Layers.Layer2.Meta.SessionMetaModel
				();
			if (isLocal)
			{
				// In local mode, we must not reload the meta model as there is no
				// concurrent access
				lastCommitedMetaModel = sessionMetaModel;
			}
			else
			{
				// In ClientServer mode, re-read the meta-model from the database
				// base to get last update.
				lastCommitedMetaModel = session.GetStorageEngine().GetObjectReader().ReadMetaModel
					(lastCommitedMetaModel, false);
			}
			// Gets the classes that have changed (that have modified ,deleted or
			// inserted objects)
			System.Collections.Generic.IEnumerator<NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
				> cis = sessionMetaModel.GetChangedClassInfo().GetEnumerator();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo newCi = null;
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo lastCommittedCI = null;
			NeoDatis.Odb.Core.Layers.Layer3.IObjectWriter writer = session.GetStorageEngine()
				.GetObjectWriter();
			NeoDatis.Odb.OID lastCommittedObjectOIDOfThisTransaction = null;
			NeoDatis.Odb.OID lastCommittedObjectOIDOfPrevTransaction = null;
			// for all changes between old and new meta model
			while (cis.MoveNext())
			{
				newCi = cis.Current;
				if (lastCommitedMetaModel.ExistClass(newCi.GetFullClassName()))
				{
					// The last CI represents the last committed meta model of the
					// database
					lastCommittedCI = lastCommitedMetaModel.GetClassInfoFromId(newCi.GetId());
					// Just be careful to keep track of current CI committed zone
					// deleted objects
					lastCommittedCI.GetCommitedZoneInfo().SetNbDeletedObjects(newCi.GetCommitedZoneInfo
						().GetNbDeletedObjects());
				}
				else
				{
					lastCommittedCI = newCi;
				}
				lastCommittedObjectOIDOfThisTransaction = newCi.GetCommitedZoneInfo().last;
				lastCommittedObjectOIDOfPrevTransaction = lastCommittedCI.GetCommitedZoneInfo().last;
				NeoDatis.Odb.OID lastCommittedObjectOID = lastCommittedObjectOIDOfPrevTransaction;
				// If some object have been created then
				if (lastCommittedObjectOIDOfPrevTransaction != null)
				{
					// Checks if last object of committed meta model has not been
					// deleted
					if (session.GetCache().IsDeleted(lastCommittedObjectOIDOfPrevTransaction))
					{
						// TODO This is wrong: if a committed transaction deleted a
						// committed object and creates x new
						// objects, then all these new objects will be lost:
						// if it has been deleted then use the last object of the
						// session class info
						lastCommittedObjectOID = lastCommittedObjectOIDOfThisTransaction;
						newCi.GetCommitedZoneInfo().last = lastCommittedObjectOID;
					}
				}
				// Connect Unconnected zone to connected zone
				// make next oid of last committed object point to first
				// uncommitted object
				// make previous oid of first uncommitted object point to
				// last committed object
				if (lastCommittedObjectOID != null && newCi.GetUncommittedZoneInfo().HasObjects())
				{
					if (newCi.GetCommitedZoneInfo().HasObjects())
					{
						// these 2 updates are executed directly without
						// transaction, because
						// We are in the commit process.
						writer.UpdateNextObjectFieldOfObjectInfo(lastCommittedObjectOID, newCi.GetUncommittedZoneInfo
							().first, false);
						writer.UpdatePreviousObjectFieldOfObjectInfo(newCi.GetUncommittedZoneInfo().first
							, lastCommittedObjectOID, false);
					}
					else
					{
						// Committed zone has 0 object
						writer.UpdatePreviousObjectFieldOfObjectInfo(newCi.GetUncommittedZoneInfo().first
							, null, false);
					}
				}
				// The number of committed objects must be updated with the number
				// of the last committed CI because a transaction may have been
				// committed changing this number.
				// Notice that the setNbObjects receive the full CommittedCIZoneInfo
				// object
				// because it will set the number of objects and the number of
				// deleted objects
				newCi.GetCommitedZoneInfo().SetNbObjects(lastCommittedCI.GetCommitedZoneInfo());
				// and don't forget to set the deleted objects
				// This sets the number of objects, the first object OID and the
				// last object OID
				newCi = BuildClassInfoForCommit(newCi);
				writer.UpdateInstanceFieldsOfClassInfo(newCi, false);
				if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
				{
					NeoDatis.Tool.DLogger.Debug("Analysing class " + newCi.GetFullClassName());
					NeoDatis.Tool.DLogger.Debug("\t-Commited CI   = " + newCi);
					NeoDatis.Tool.DLogger.Debug("\t-connect last commited object with oid " + lastCommittedObjectOID
						 + " to first uncommited object " + newCi.GetUncommittedZoneInfo().first);
					NeoDatis.Tool.DLogger.Debug("\t-Commiting new Number of objects = " + newCi.GetNumberOfObjects
						());
				}
			}
			sessionMetaModel.ResetChangedClasses();
			// To guarantee integrity after commit, the meta model is set to null
			// If the user continues using odb instance after commit the meta model
			// will be lazy-reloaded. Only for Client Server mode
			if (!isLocal)
			{
				session.SetMetaModel(null);
			}
		}

		/// <summary>Shift all unconnected infos to connected (committed) infos</summary>
		/// <param name="classInfo"></param>
		/// <returns>The updated class info</returns>
		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo BuildClassInfoForCommit
			(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo classInfo)
		{
			long nbObjects = classInfo.GetNumberOfObjects();
			classInfo.GetCommitedZoneInfo().SetNbObjects(nbObjects);
			if (classInfo.GetCommitedZoneInfo().first != null)
			{
			}
			else
			{
				// nothing to change
				classInfo.GetCommitedZoneInfo().first = classInfo.GetUncommittedZoneInfo().first;
			}
			if (classInfo.GetUncommittedZoneInfo().last != null)
			{
				classInfo.GetCommitedZoneInfo().last = classInfo.GetUncommittedZoneInfo().last;
			}
			// Resets the unconnected zone info
			classInfo.GetUncommittedZoneInfo().Set(new NeoDatis.Odb.Core.Layers.Layer2.Meta.CIZoneInfo
				(classInfo, null, null, 0));
			return classInfo;
		}

		/// <exception cref="System.IO.IOException"></exception>
		/// <exception cref="System.TypeLoadException"></exception>
		public static NeoDatis.Odb.Impl.Core.Transaction.DefaultTransaction Read(string fileName
			)
		{
			NeoDatis.Tool.Wrappers.IO.OdbFile file = new NeoDatis.Tool.Wrappers.IO.OdbFile(fileName
				);
			if (!file.Exists())
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.FileNotFound
					.AddParameter(fileName));
			}
			// @TODO check this
			NeoDatis.Odb.Impl.Core.Transaction.DefaultTransaction transaction = new NeoDatis.Odb.Impl.Core.Transaction.DefaultTransaction
				(null, fileName);
			transaction.LoadWriteActions(fileName, false);
			return transaction;
		}

		public virtual void LoadWriteActions(bool apply)
		{
			LoadWriteActions(GetName(), apply);
		}

		public virtual void LoadWriteActions(string filename, bool apply)
		{
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
			{
				NeoDatis.Tool.DLogger.Debug("Load write actions of " + filename);
			}
			NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction wa = null;
			try
			{
				CheckFileAccess(false, filename);
				fsi.UseBuffer(true);
				fsi.SetReadPosition(0);
				isCommited = fsi.ReadByte() == 1;
				creationDateTime = fsi.ReadLong();
				long totalNumberOfWriteActions = fsi.ReadLong();
				if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
				{
					NeoDatis.Tool.DLogger.Info(writeActions.Count + " write actions in file");
				}
				for (int i = 0; i < totalNumberOfWriteActions; i++)
				{
					wa = NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction.Read(fsi, i + 1);
					if (apply)
					{
						wa.ApplyTo(fsiToApplyWriteActions, i + 1);
						wa.Clear();
					}
					else
					{
						AddWriteAction(wa, false);
					}
				}
				if (apply)
				{
					fsiToApplyWriteActions.Flush();
				}
			}
			finally
			{
			}
		}

		/// <exception cref="System.IO.IOException"></exception>
		/// <exception cref="System.TypeLoadException"></exception>
		public virtual void LoadWriteActionsBackwards(string filename, bool apply)
		{
			int executedWriteAction = 0;
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
			{
				NeoDatis.Tool.DLogger.Debug("Load write actions of " + filename);
			}
			NeoDatis.Odb.Core.Transaction.IWriteAction wa = null;
			try
			{
				CheckFileAccess(false, filename);
				fsi.UseBuffer(true);
				fsi.SetReadPosition(0);
				isCommited = fsi.ReadByte() == 1;
				creationDateTime = fsi.ReadLong();
				System.Collections.Generic.IDictionary<long, long> writtenPositions = null;
				if (apply)
				{
					writtenPositions = new NeoDatis.Tool.Wrappers.Map.OdbHashMap<long, long>();
				}
				long position = System.Convert.ToInt64(-1);
				int i = numberOfWriteActions;
				long previousWriteActionPosition = fsi.GetLength();
				while (i > 0)
				{
					// Sets the position 8 bytes backwards
					fsi.SetReadPosition(previousWriteActionPosition - NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
						.Long.GetSize());
					// And then the read a long, this will be the previous write
					// action position
					previousWriteActionPosition = fsi.ReadLong();
					// Then sets the read position to read the write action
					fsi.SetReadPosition(previousWriteActionPosition);
					wa = NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction.Read(fsi, i + 1);
					if (apply)
					{
						position = wa.GetPosition();
						if (writtenPositions[position] != null)
						{
							// It has already been written something more recent at
							// this position, do not write again
							i--;
							continue;
						}
						wa.ApplyTo(fsiToApplyWriteActions, i + 1);
						writtenPositions.Add(position, position);
						executedWriteAction++;
					}
					else
					{
						AddWriteAction(wa, false);
					}
					i--;
				}
				if (apply)
				{
					fsiToApplyWriteActions.Flush();
					if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
					{
						NeoDatis.Tool.DLogger.Debug("Total Write actions : " + i + " / position cache = "
							 + writtenPositions.Count);
					}
					NeoDatis.Tool.DLogger.Info("Total write actions = " + numberOfWriteActions + " : executed = "
						 + executedWriteAction);
					writtenPositions.Clear();
					writtenPositions = null;
				}
			}
			finally
			{
			}
		}

		/// <summary>deletes the transaction file</summary>
		/// <exception cref="System.IO.IOException">System.IO.IOException</exception>
		protected virtual void Delete()
		{
		}

		// The delete is done automatically by underlying api
		public override string ToString()
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			buffer.Append("state=").Append(isCommited).Append(" | creation=").Append(creationDateTime
				).Append(" | write actions numbers=").Append(numberOfWriteActions);
			return buffer.ToString();
		}

		private void ApplyTo()
		{
			int realWriteNumber = 0;
			int noPointerWA = 0;
			if (!isCommited)
			{
				NeoDatis.Tool.DLogger.Info("can not execute a transaction that is not confirmed");
				return;
			}
			if (hasAllWriteActionsInMemory)
			{
				for (int i = 0; i < writeActions.Count; i++)
				{
					NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction wa = (NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
						)writeActions[i];
					wa.ApplyTo(fsiToApplyWriteActions, i + 1);
					wa.Clear();
				}
				fsiToApplyWriteActions.Flush();
			}
			else
			{
				LoadWriteActions(true);
				fsiToApplyWriteActions.Flush();
			}
		}

		public virtual void SetFsiToApplyWriteActions(NeoDatis.Odb.Core.Layers.Layer3.Engine.IFileSystemInterface
			 fsi)
		{
			this.fsiToApplyWriteActions = fsi;
		}

		public virtual bool IsArchiveLog()
		{
			return archiveLog;
		}

		public virtual void SetArchiveLog(bool archiveLog)
		{
			this.archiveLog = archiveLog;
		}

		/// <returns>Returns the numberOfWriteActions.</returns>
		public virtual int GetNumberOfWriteActions()
		{
			if (currentWriteAction != null && !currentWriteAction.IsEmpty())
			{
				return numberOfWriteActions + 1;
			}
			return numberOfWriteActions;
		}

		/// <exception cref="System.IO.IOException"></exception>
		public virtual NeoDatis.Odb.Core.Layers.Layer3.Engine.IFileSystemInterface GetFsi
			()
		{
			if (fsi == null)
			{
				CheckFileAccess(!readOnlyMode);
			}
			return fsi;
		}

		/// <summary>Set the write position (position in main database file).</summary>
		/// <remarks>
		/// Set the write position (position in main database file). This is used to
		/// know if the next write can be appended to the previous one (in the same
		/// current Write Action) or not.
		/// </remarks>
		/// <param name="position"></param>
		public virtual void SetWritePosition(long position)
		{
			if (position != this.currentWritePositionInWA)
			{
				this.currentWritePositionInWA = position;
				if (currentWriteAction != null)
				{
					AddWriteAction(currentWriteAction);
				}
				this.currentWriteAction = new NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
					(position);
			}
			else
			{
				if (currentWriteAction == null)
				{
					this.currentWriteAction = new NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
						(position);
					this.currentWritePositionInWA = position;
				}
			}
		}

		public virtual void ManageWriteAction(long position, byte[] bytes)
		{
			if (this.currentWritePositionInWA == position)
			{
				if (currentWriteAction == null)
				{
					currentWriteAction = provider.GetWriteAction(position, null);
				}
				currentWriteAction.AddBytes(bytes);
				this.currentWritePositionInWA += bytes.Length;
			}
			else
			{
				if (currentWriteAction != null)
				{
					AddWriteAction(currentWriteAction);
				}
				this.currentWriteAction = provider.GetWriteAction(position, bytes);
				this.currentWritePositionInWA = position + bytes.Length;
			}
		}
	}
}
