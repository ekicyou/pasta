using NeoDatis.Odb.Core.Layers.Layer2.Meta;
using NeoDatis.Odb.Impl.Core.Btree;
using NeoDatis.Tool.Wrappers.List;
using NeoDatis.Odb.Core;
using NeoDatis.Btree;
using NeoDatis.Odb.Core.Query;
using NeoDatis.Odb.Impl.Core.Query.Criteria;
using NeoDatis.Odb.Core.Query.Criteria;
using NeoDatis.Tool;
using NeoDatis.Odb.Impl.Core.Layers.Layer3.Block;
using NeoDatis.Odb.Core.Transaction;
using NeoDatis.Odb.Core.Query.Execution;
using NeoDatis.Tool.Wrappers;
using NeoDatis.Odb.Core.Oid;
using System.Collections.Generic;
using NeoDatis.Tool.Wrappers.Map;
namespace NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine
{
	/// <summary>Manage all IO Reading</summary>
	/// <author>olivier smadja</author>
	public class ObjectReader : NeoDatis.Odb.Core.Layers.Layer3.IObjectReader
	{
		public static long timeToGetObjectFromId = 0;

		public static long calls = 0;

		public static readonly string LogId = "ObjectReader";

		private static readonly string LogIdDebug = "ObjectReader.debug";

		/// <summary>The storage engine</summary>
		public NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine storageEngine;

		/// <summary>To hold block number.</summary>
		/// <remarks>
		/// To hold block number. ODB compute the block number from the oid (as one
		/// block has 1000 oids), then it has to search the position of the block
		/// number! This cache is used to keep track of the positions of the block
		/// positions The key is the block number(Long) and the value the position
		/// (Long)
		/// </remarks>
		private System.Collections.Generic.IDictionary<long, long> blockPositions;

		/// <summary>The fsi is the object that knows how to write and read native types</summary>
		private NeoDatis.Odb.Core.Layers.Layer3.Engine.IFileSystemInterface fsi;

		/// <summary>A local variable to monitor object recursion</summary>
		private int currentDepth;

		/// <summary>to build instances</summary>
		private NeoDatis.Odb.Core.Layers.Layer2.Instance.IInstanceBuilder instanceBuilder;

		/// <summary>to boost class fetch</summary>
		private NeoDatis.Odb.Core.Layers.Layer2.Instance.IClassPool classPool;

		protected NeoDatis.Odb.Core.Layers.Layer3.Engine.IByteArrayConverter byteArrayConverter;

		protected NeoDatis.Odb.Core.Trigger.ITriggerManager triggerManager;

		/// <summary>A small method for indentation</summary>
		public virtual string DepthToSpaces()
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			for (int i = 0; i < currentDepth; i++)
			{
				buffer.Append("  ");
			}
			return buffer.ToString();
		}

		/// <summary>The constructor</summary>
		/// <param name="engine"></param>
		/// <param name="triggerManager"></param>
		public ObjectReader(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine)
		{
			this.storageEngine = engine;
			this.fsi = engine.GetObjectWriter().GetFsi();
			blockPositions = new NeoDatis.Tool.Wrappers.Map.OdbHashMap<long, long>();
			this.instanceBuilder = BuildInstanceBuilder();
			this.classPool = NeoDatis.Odb.OdbConfiguration.GetCoreProvider().GetClassPool();
			this.byteArrayConverter = NeoDatis.Odb.OdbConfiguration.GetCoreProvider().GetByteArrayConverter
				();
			this.triggerManager = storageEngine.GetTriggerManager();
		}

		protected virtual NeoDatis.Odb.Core.Layers.Layer2.Instance.IInstanceBuilder BuildInstanceBuilder
			()
		{
			return NeoDatis.Odb.OdbConfiguration.GetCoreProvider().GetLocalInstanceBuilder(storageEngine
				);
		}

		/// <summary>Read the version of the database file</summary>
		protected virtual int ReadVersion()
		{
			fsi.SetReadPosition(NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.DatabaseHeaderVersionPosition);
			return fsi.ReadInt();
		}

		/// <summary>Read the encryption flag of the database file</summary>
		protected virtual bool ReadEncryptionFlag()
		{
			fsi.SetReadPosition(NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.DatabaseHeaderUseEncryptionPosition);
			byte b = fsi.ReadByte();
			return b == NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant.WithEncryption;
		}

		/// <summary>Read the replication flag of the database file</summary>
		protected virtual bool ReadReplicationFlag()
		{
			fsi.SetReadPosition(NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.DatabaseHeaderUseReplicationPosition);
			byte b = fsi.ReadByte();
			return b == NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant.WithEncryption;
		}

		/// <summary>Read the last transaction id</summary>
		protected virtual NeoDatis.Odb.TransactionId ReadLastTransactionId(NeoDatis.Odb.DatabaseId
			 databaseId)
		{
			fsi.SetReadPosition(NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.DatabaseHeaderLastTransactionId);
			long[] id = new long[2];
			id[0] = fsi.ReadLong();
			id[1] = fsi.ReadLong();
			return new NeoDatis.Odb.Impl.Core.Oid.TransactionIdImpl(databaseId, id[0], id[1]);
		}

		/// <summary>Reads the number of classes in database file</summary>
		protected virtual long ReadNumberOfClasses()
		{
			fsi.SetReadPosition(NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.DatabaseHeaderNumberOfClassesPosition);
			return fsi.ReadLong();
		}

		/// <summary>Reads the first class OID</summary>
		protected virtual long ReadFirstClassOid()
		{
			fsi.SetReadPosition(NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.DatabaseHeaderFirstClassOid);
			return fsi.ReadLong();
		}

		/// <summary>Reads the status of the last odb close</summary>
		protected virtual bool ReadLastODBCloseStatus()
		{
			fsi.SetReadPosition(NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.DatabaseHeaderLastCloseStatusPosition);
			return fsi.ReadBoolean("last odb status");
		}

		/// <summary>Reads the database character encoding</summary>
		protected virtual string ReadDatabaseCharacterEncoding()
		{
			fsi.SetReadPosition(NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.DatabaseHeaderDatabaseCharacterEncodingPosition);
			return fsi.ReadString(false);
		}

		/// <summary>see http://wiki.neodatis.org/odb-file-format</summary>
		public virtual void ReadDatabaseHeader(string user, string password)
		{
			bool useEncryption = ReadEncryptionFlag();
			// Reads the version of the database file
			int version = ReadVersion();
			bool versionIsCompatible = version == NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.CurrentFileFormatVersion;
			if (!versionIsCompatible)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.RuntimeIncompatibleVersion
					.AddParameter(version).AddParameter(NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
					.CurrentFileFormatVersion));
			}
			long[] databaseIdsArray = new long[4];
			databaseIdsArray[0] = fsi.ReadLong();
			databaseIdsArray[1] = fsi.ReadLong();
			databaseIdsArray[2] = fsi.ReadLong();
			databaseIdsArray[3] = fsi.ReadLong();
			NeoDatis.Odb.DatabaseId databaseId = new NeoDatis.Odb.Impl.Core.Oid.DatabaseIdImpl
				(databaseIdsArray);
			bool isReplicated = ReadReplicationFlag();
			NeoDatis.Odb.TransactionId lastTransactionId = ReadLastTransactionId(databaseId);
			// Increment transaction id
			lastTransactionId = lastTransactionId.Next();
			long nbClasses = ReadNumberOfClasses();
			long firstClassPosition = ReadFirstClassOid();
			if (nbClasses < 0)
			{
				throw new NeoDatis.Odb.CorruptedDatabaseException(NeoDatis.Odb.Core.NeoDatisError
					.NegativeClassNumberInHeader.AddParameter(nbClasses).AddParameter(firstClassPosition
					));
			}
			bool lastCloseStatus = ReadLastODBCloseStatus();
			string databaseCharacterEncoding = ReadDatabaseCharacterEncoding();
			fsi.SetDatabaseCharacterEncoding(databaseCharacterEncoding);
			bool hasUserAndPassword = fsi.ReadBoolean("has user&password?");
			// even if database is not user/password protected, there is a fake
			// user/password that has been written
			string userRead = fsi.ReadString(true);
			string passwordRead = fsi.ReadString(true);
			if (hasUserAndPassword)
			{
				string encryptedPassword = NeoDatis.Odb.Impl.Tool.Cryptographer.Encrypt(password);
				if (!userRead.Equals(user) || !passwordRead.Equals(encryptedPassword))
				{
					throw new NeoDatis.Odb.ODBAuthenticationRuntimeException();
				}
			}
			else
			{
				if (user != null)
				{
					throw new NeoDatis.Odb.ODBAuthenticationRuntimeException();
				}
			}
			long currentBlockPosition = fsi.ReadLong("current block position");
			// Gets the current id block number
			fsi.SetReadPosition(currentBlockPosition + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.BlockIdOffsetForBlockNumber);
			int currentBlockNumber = fsi.ReadInt("current block id number");
			NeoDatis.Odb.OID maxId = NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(fsi.ReadLong
				("Block max id"));
			storageEngine.SetVersion(version);
			storageEngine.SetDatabaseId(databaseId);
			storageEngine.SetNbClasses(nbClasses);
			storageEngine.SetLastODBCloseStatus(lastCloseStatus);
			storageEngine.SetCurrentIdBlockInfos(currentBlockPosition, currentBlockNumber, maxId
				);
			storageEngine.SetCurrentTransactionId(lastTransactionId);
		}

		public virtual MetaModel ReadMetaModel(MetaModel metaModel, bool full)
		{
			OID classOID = null;
			ClassInfo classInfo = null;
			long nbClasses = ReadNumberOfClasses();
			if (nbClasses == 0)
			{
				return metaModel;
			}
			// Set the cursor Where We Can Find The First Class info OID
			fsi.SetReadPosition(StorageEngineConstant.DatabaseHeaderFirstClassOid);
			classOID = NeoDatis.Odb.Core.Oid.OIDFactory.BuildClassOID(ReadFirstClassOid());
			// read headers
			for (int i = 0; i < nbClasses; i++)
			{
				classInfo = ReadClassInfoHeader(classOID);
				if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
				{
					NeoDatis.Tool.DLogger.Debug(DepthToSpaces() + "Reading class header for " + classInfo
						.GetFullClassName() + " - oid = " + classOID + " prevOid=" + classInfo.GetPreviousClassOID
						() + " - nextOid=" + classInfo.GetNextClassOID());
				}
				metaModel.AddClass(classInfo);
				classOID = classInfo.GetNextClassOID();
			}
			if (!full)
			{
				return metaModel;
			}
			IOdbList<ClassInfo> allClasses = metaModel.GetAllClasses();
			System.Collections.IEnumerator iterator = allClasses.GetEnumerator();
			ClassInfo tempCi = null;
			// Read class info bodies
			while (iterator.MoveNext())
			{
				tempCi = (NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo)iterator.Current;
				try
				{
					classInfo = ReadClassInfoBody(tempCi);
				}
				catch (NeoDatis.Odb.ODBRuntimeException e)
				{
					e.AddMessageHeader("Error while reading the class info body of " + tempCi);
					throw;
				}
				if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
				{
					NeoDatis.Tool.DLogger.Debug(DepthToSpaces() + "Reading class body for " + classInfo
						.GetFullClassName());
				}
			}
			// No need to add it to metamodel, it is already in it.
			// metaModel.addClass(classInfo);
			// Read last object of each class
			iterator = allClasses.GetEnumerator();
			while (iterator.MoveNext())
			{
				classInfo = (ClassInfo)iterator.Current;
				if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
				{
					NeoDatis.Tool.DLogger.Debug(DepthToSpaces() + "Reading class info last instance "
						 + classInfo.GetFullClassName());
				}
				if (classInfo.GetCommitedZoneInfo().HasObjects())
				{
					// TODO Check if must use true or false in return object
					// parameter
					try
					{
						// Retrieve the object by oid instead of position
						NeoDatis.Odb.OID oid = classInfo.GetCommitedZoneInfo().last;
						classInfo.SetLastObjectInfoHeader(ReadObjectInfoHeaderFromOid(oid, true));
					}
					catch (NeoDatis.Odb.ODBRuntimeException e)
					{
						throw new ODBRuntimeException(NeoDatisError.MetamodelReadingLastObject.AddParameter(classInfo.GetFullClassName()).AddParameter(classInfo.GetCommitedZoneInfo().last), e);
					}
				}
			}
			IOdbList<ClassInfoIndex> indexes = null;
			NeoDatis.Btree.IBTreePersister persister = null;
			ClassInfoIndex cii = null;
			IQuery queryClassInfo = null;
			IBTree btree = null;
			storageEngine.ResetCommitListeners();
			// Read class info indexes
			iterator = allClasses.GetEnumerator();
			while (iterator.MoveNext())
			{
				classInfo = (ClassInfo)iterator.Current;
				indexes = new OdbArrayList<ClassInfoIndex>();
				queryClassInfo = new CriteriaQuery(typeof(ClassInfoIndex), Where.Equal("classInfoId", classInfo.GetId()
					));
				Objects<ClassInfoIndex> classIndexes = GetObjects<ClassInfoIndex>(queryClassInfo, true, -1, -1);
				indexes.AddAll(classIndexes);
				// Sets the btree persister
				for (int j = 0; j < indexes.Count; j++)
				{
					cii = indexes[j];
					persister = new LazyODBBTreePersister(storageEngine);
					btree = cii.GetBTree();
					btree.SetPersister(persister);
					btree.GetRoot().SetBTree(btree);
				}
				if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
				{
					DLogger.Debug(DepthToSpaces() + "Reading indexes for " + classInfo.	GetFullClassName() + " : " + indexes.Count + " indexes");
				}
				classInfo.SetIndexes(indexes);
			}
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
			{
				DLogger.Debug(DepthToSpaces() + "Current Meta Model is :" + metaModel);
			}
			return metaModel;
		}

		/// <summary>Read the class info header with the specific oid</summary>
		/// <param name="startPosition"></param>
		/// <returns>The read class info object @</returns>
		protected virtual ClassInfo ReadClassInfoHeader(OID classInfoOid)
		{
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
			{
				NeoDatis.Tool.DLogger.Debug(DepthToSpaces() + "Reading new Class info Header with oid "
					 + classInfoOid);
			}
			long classInfoPosition = GetObjectPositionFromItsOid(classInfoOid, true, true);
			fsi.SetReadPosition(classInfoPosition);
			int blockSize = fsi.ReadInt("class info block size");
			byte blockType = fsi.ReadByte("class info block type");
			if (!BlockTypes.IsClassHeader(blockType
				))
			{
				throw new ODBRuntimeException(NeoDatisError.WrongTypeForBlockType.AddParameter("Class Header").AddParameter(blockType).AddParameter(classInfoPosition
					));
			}
			byte classInfoCategory = fsi.ReadByte("class info category");
			ClassInfo classInfo = new ClassInfo();
			classInfo.SetClassCategory(classInfoCategory);
			classInfo.SetPosition(classInfoPosition);
			classInfo.SetId(NeoDatis.Odb.Core.Oid.OIDFactory.BuildClassOID(fsi.ReadLong()));
			classInfo.SetBlockSize(blockSize);
			classInfo.SetPreviousClassOID(ReadOid("prev class oid"));
			classInfo.SetNextClassOID(ReadOid("next class oid"));
			classInfo.GetOriginalZoneInfo().SetNbObjects(fsi.ReadLong());
			classInfo.GetOriginalZoneInfo().first = ReadOid("ci first object oid");
			classInfo.GetOriginalZoneInfo().last = ReadOid("ci last object oid");
			classInfo.GetCommitedZoneInfo().Set(classInfo.GetOriginalZoneInfo());
			classInfo.SetFullClassName(fsi.ReadString(false));
			// FIXME : Extract extra info : c# compatibility
			classInfo.SetExtraInfo(string.Empty);
			classInfo.SetMaxAttributeId(fsi.ReadInt());
			classInfo.SetAttributesDefinitionPosition(fsi.ReadLong());
			// FIXME Convert block size to long ??
			int realBlockSize = (int)(fsi.GetPosition() - classInfoPosition);
			if (blockSize != realBlockSize)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.WrongBlockSize
					.AddParameter(blockSize).AddParameter(realBlockSize).AddParameter(classInfoPosition
					));
			}
			return classInfo;
		}

		private NeoDatis.Odb.OID DecodeOid(byte[] bytes, int offset)
		{
			long oid = byteArrayConverter.ByteArrayToLong(bytes, offset);
			if (oid == -1)
			{
				return null;
			}
			return NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(oid);
		}

		private NeoDatis.Odb.OID ReadOid(string label)
		{
			long oid = fsi.ReadLong(label);
			if (oid == -1)
			{
				return null;
			}
			return NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(oid);
		}

		/// <summary>Reads the body of a class info</summary>
		/// <param name="classInfo">The class info to be read with already read header</param>
		/// <returns>The read class info @</returns>
		protected virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ReadClassInfoBody
			(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo classInfo)
		{
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
			{
				NeoDatis.Tool.DLogger.Debug(DepthToSpaces() + "Reading new Class info Body at " +
					 classInfo.GetAttributesDefinitionPosition());
			}
			fsi.SetReadPosition(classInfo.GetAttributesDefinitionPosition());
			int blockSize = fsi.ReadInt();
			byte blockType = fsi.ReadByte();
			if (!NeoDatis.Odb.Impl.Core.Layers.Layer3.Block.BlockTypes.IsClassBody(blockType))
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.WrongTypeForBlockType
					.AddParameter("Class Body").AddParameter(blockType).AddParameter(classInfo.GetAttributesDefinitionPosition
					()));
			}
			// TODO This should be a short instead of long
			long nbAttributes = fsi.ReadLong();
			NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassAttributeInfo
				> attributes = new NeoDatis.Tool.Wrappers.List.OdbArrayList<NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassAttributeInfo
				>((int)nbAttributes);
			for (int i = 0; i < nbAttributes; i++)
			{
				attributes.Add(ReadClassAttributeInfo());
			}
			classInfo.SetAttributes(attributes);
			// FIXME Convert blocksize to long ??
			int realBlockSize = (int)(fsi.GetPosition() - classInfo.GetAttributesDefinitionPosition
				());
			if (blockSize != realBlockSize)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.WrongBlockSize
					.AddParameter(blockSize).AddParameter(realBlockSize).AddParameter(classInfo.GetAttributesDefinitionPosition
					()));
			}
			return classInfo;
		}

		public virtual NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoIndex
			> ReadClassInfoIndexesAt(long position, NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			 classInfo)
		{
			NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoIndex
				> indexes = new NeoDatis.Tool.Wrappers.List.OdbArrayList<NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoIndex
				>();
			fsi.SetReadPosition(position);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoIndex cii = null;
			long previousIndexPosition = -1;
			long nextIndexPosition = position;
			byte blockType = 0;
			int blockSize = -1;
			int nbAttributes = -1;
			int[] attributeIds = null;
			do
			{
				cii = new NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoIndex();
				fsi.SetReadPosition(nextIndexPosition);
				blockSize = fsi.ReadInt("block size");
				blockType = fsi.ReadByte("block type");
				if (!NeoDatis.Odb.Impl.Core.Layers.Layer3.Block.BlockTypes.IsIndex(blockType))
				{
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.WrongTypeForBlockType
						.AddParameter(NeoDatis.Odb.Impl.Core.Layers.Layer3.Block.BlockTypes.BlockTypeIndex
						).AddParameter(blockType).AddParameter(position).AddParameter("while reading indexes for "
						 + classInfo.GetFullClassName()));
				}
				previousIndexPosition = fsi.ReadLong("prev index pos");
				nextIndexPosition = fsi.ReadLong("next index pos");
				cii.SetName(fsi.ReadString(false, "Index name"));
				cii.SetUnique(fsi.ReadBoolean("index is unique"));
				cii.SetStatus(fsi.ReadByte("index status"));
				cii.SetCreationDate(fsi.ReadLong("creation date"));
				cii.SetLastRebuild(fsi.ReadLong("last rebuild"));
				nbAttributes = fsi.ReadInt("number of fields");
				attributeIds = new int[nbAttributes];
				for (int j = 0; j < nbAttributes; j++)
				{
					attributeIds[j] = fsi.ReadInt("attr id");
				}
				cii.SetAttributeIds(attributeIds);
				indexes.Add(cii);
			}
			while (nextIndexPosition != -1);
			return indexes;
		}

		/// <summary>Read an attribute of a class at the current position</summary>
		/// <returns>The ClassAttributeInfo description of the class attribute @</returns>
		private NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassAttributeInfo ReadClassAttributeInfo
			()
		{
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassAttributeInfo cai = new NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassAttributeInfo
				();
			int attributeId = fsi.ReadInt();
			bool isNative = fsi.ReadBoolean();
			if (isNative)
			{
				int attributeTypeId = fsi.ReadInt();
				NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType type = NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
					.GetFromId(attributeTypeId);
				// if it is an array, read also the subtype
				if (type.IsArray())
				{
					type = type.Copy();
					int subTypeId = fsi.ReadInt();
					NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType subType = NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
						.GetFromId(subTypeId);
					if (subType.IsNonNative())
					{
						subType = subType.Copy();
						subType.SetName(storageEngine.GetSession(true).GetMetaModel().GetClassInfoFromId(
							NeoDatis.Odb.Core.Oid.OIDFactory.BuildClassOID(fsi.ReadLong())).GetFullClassName
							());
					}
					type.SetSubType(subType);
				}
				cai.SetAttributeType(type);
				// For enum, we get the class info id of the enum class
				if (type.IsEnum())
				{
					long classInfoId = fsi.ReadLong();
					NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel metaModel = storageEngine.GetSession
						(true).GetMetaModel();
					cai.SetFullClassName(metaModel.GetClassInfoFromId(NeoDatis.Odb.Core.Oid.OIDFactory
						.BuildClassOID(classInfoId)).GetFullClassName());
					// For enum, we need to create a new type just to set the real enum class name
					type = type.Copy();
					type.SetName(cai.GetFullClassname());
					cai.SetAttributeType(type);
				}
				else
				{
					cai.SetFullClassName(cai.GetAttributeType().GetName());
				}
			}
			else
			{
				// This is a non native, gets the id of the type and gets it from
				// meta-model
				NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel metaModel = storageEngine.GetSession
					(true).GetMetaModel();
				long typeId = fsi.ReadLong();
				cai.SetFullClassName(metaModel.GetClassInfoFromId(NeoDatis.Odb.Core.Oid.OIDFactory
					.BuildClassOID(typeId)).GetFullClassName());
				cai.SetClassInfo(metaModel.GetClassInfo(cai.GetFullClassname(), true));
				cai.SetAttributeType(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.GetFromName(cai
					.GetFullClassname()));
			}
			cai.SetName(fsi.ReadString(false));
			cai.SetIndex(fsi.ReadBoolean());
			cai.SetId(attributeId);
			return cai;
		}

		/// <summary>Reads an object at the specific position</summary>
		/// <param name="position">The position to read</param>
		/// <param name="useCache">To indicate if cache must be used</param>
		/// <param name="To">indicate if an instance must be return of just the meta info</param>
		/// <returns>The object with position @</returns>
		public virtual object ReadNonNativeObjectAtPosition(long position, bool useCache, 
			bool returnInstance)
		{
			// First reads the object info - which is a meta representation of the
			// object
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoi = ReadNonNativeObjectInfoFromPosition
				(null, null, position, useCache, returnInstance);
			if (nnoi.IsDeletedObject())
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ObjectIsMarkedAsDeletedForPosition
					.AddParameter(position));
			}
			if (!returnInstance)
			{
				return nnoi;
			}
			// Then converts it to the real object
			object o = instanceBuilder.BuildOneInstance(nnoi);
			return o;
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo ReadObjectInfo
			(long objectIdentification, bool useCache, bool returnObjects)
		{
			// If object identification is negative, it is an oid.
			if (objectIdentification < 0)
			{
				NeoDatis.Odb.OID oid = NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(-objectIdentification
					);
				return ReadNonNativeObjectInfoFromOid(null, oid, useCache, returnObjects);
			}
			return ReadObjectInfoFromPosition(null, objectIdentification, useCache, returnObjects
				);
		}

		/// <summary>
		/// Reads the pointers(ids or positions) of an object that has the specific
		/// oid
		/// </summary>
		/// <param name="oid">The oid of the object we want to read the pointers</param>
		/// <returns>The ObjectInfoHeader @</returns>
		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader ReadObjectInfoHeaderFromOid
			(NeoDatis.Odb.OID oid, bool useCache)
		{
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader oih = null;
			if (useCache)
			{
				oih = GetSession().GetCache().GetObjectInfoHeaderFromOid(oid, false);
				if (oih != null)
				{
					return oih;
				}
			}
			long position = GetObjectPositionFromItsOid(oid, useCache, true);
			return ReadObjectInfoHeaderFromPosition(oid, position, useCache);
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader ReadObjectInfoHeaderFromPosition
			(NeoDatis.Odb.OID oid, long position, bool useCache)
		{
			NeoDatis.Odb.OID classInfoId = null;
			if (position > fsi.GetLength())
			{
				throw new NeoDatis.Odb.CorruptedDatabaseException(NeoDatis.Odb.Core.NeoDatisError
					.InstancePositionOutOfFile.AddParameter(position).AddParameter(fsi.GetLength()));
			}
			if (position < 0)
			{
				throw new NeoDatis.Odb.CorruptedDatabaseException(NeoDatis.Odb.Core.NeoDatisError
					.InstancePositionIsNegative.AddParameter(position).AddParameter(oid.ToString()));
			}
			// adds an integer because, we pull the block size
			fsi.SetReadPosition(position + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Integer
				.GetSize());
			byte blockType = fsi.ReadByte("object block type");
			if (NeoDatis.Odb.Impl.Core.Layers.Layer3.Block.BlockTypes.IsNonNative(blockType))
			{
				// compute the number of bytes to read
				// OID + ClassOid + PrevOid + NextOid + createDate + update Date + objectVersion + objectRefPointer + isSync + nbAttributes
				// Long + Long +    Long    +  Long    + Long       + Long       +   int         +   Long            + Bool    + Int       
				// atsize = ODBType.SIZE_OF_INT+ODBType.SIZE_OF_LONG;
				int tsize = 7 * NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.SizeOfLong + 2 * NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
					.SizeOfInt + 1 * NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.SizeOfBool;
				byte[] abytes = fsi.ReadBytes(tsize);
				NeoDatis.Odb.OID readOid = DecodeOid(abytes, 0);
				// oid can be -1 (if was not set),in this case there is no way to
				// check
				if (oid != null && readOid.CompareTo(oid) != 0)
				{
					throw new NeoDatis.Odb.CorruptedDatabaseException(NeoDatis.Odb.Core.NeoDatisError
						.WrongOidAtPosition.AddParameter(oid).AddParameter(position).AddParameter(readOid
						));
				}
				// If oid is not defined, uses the one that has been read
				if (oid == null)
				{
					oid = readOid;
				}
				// It is a non native object
				classInfoId = NeoDatis.Odb.Core.Oid.OIDFactory.BuildClassOID(byteArrayConverter.ByteArrayToLong
					(abytes, 8));
				NeoDatis.Odb.OID prevObjectOID = DecodeOid(abytes, 16);
				NeoDatis.Odb.OID nextObjectOID = DecodeOid(abytes, 24);
				long creationDate = byteArrayConverter.ByteArrayToLong(abytes, 32);
				long updateDate = byteArrayConverter.ByteArrayToLong(abytes, 40);
				int objectVersion = byteArrayConverter.ByteArrayToInt(abytes, 48);
				long objectReferencePointer = byteArrayConverter.ByteArrayToLong(abytes, 52);
				bool isSynchronized = byteArrayConverter.ByteArrayToBoolean(abytes, 60);
				// Now gets info about attributes
				int nbAttributesRead = byteArrayConverter.ByteArrayToInt(abytes, 61);
				// Now gets an array with the identification all attributes (can be
				// positions(for native objects) or ids(for non native objects))
				long[] attributesIdentification = new long[nbAttributesRead];
				int[] attributeIds = new int[nbAttributesRead];
				int atsize = NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.SizeOfInt + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
					.SizeOfLong;
				// Reads the bytes and then convert to values
				byte[] bytes = fsi.ReadBytes(nbAttributesRead * atsize);
				for (int i = 0; i < nbAttributesRead; i++)
				{
					attributeIds[i] = byteArrayConverter.ByteArrayToInt(bytes, i * atsize);
					attributesIdentification[i] = byteArrayConverter.ByteArrayToLong(bytes, i * atsize
						 + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.SizeOfInt);
				}
				NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader oip = new NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader
					(position, prevObjectOID, nextObjectOID, classInfoId, attributesIdentification, 
					attributeIds);
				oip.SetObjectVersion(objectVersion);
				oip.SetCreationDate(creationDate);
				oip.SetUpdateDate(updateDate);
				oip.SetOid(oid);
				oip.SetClassInfoId(classInfoId);
				// oip.setCreationDate(creationDate);
				// oip.setUpdateDate(updateDate);
				// oip.setObjectVersion(objectVersion);
				if (useCache)
				{
					// the object info does not exist in the cache
					storageEngine.GetSession(true).GetCache().AddObjectInfo(oip);
				}
				return oip;
			}
			if (NeoDatis.Odb.Impl.Core.Layers.Layer3.Block.BlockTypes.IsPointer(blockType))
			{
				throw new NeoDatis.Odb.CorruptedDatabaseException(NeoDatis.Odb.Core.NeoDatisError
					.FoundPointer.AddParameter(oid).AddParameter(position));
			}
			throw new NeoDatis.Odb.CorruptedDatabaseException(NeoDatis.Odb.Core.NeoDatisError
				.WrongTypeForBlockType.AddParameter(NeoDatis.Odb.Impl.Core.Layers.Layer3.Block.BlockTypes
				.BlockTypeNonNativeObject).AddParameter(blockType).AddParameter(position + "/oid="
				 + oid));
		}

		/// <summary>
		/// Reads an object info(Object meta information like its type and its
		/// values) from the database file
		/// <p/>
		/// <pre>
		/// reads its type and then read all its attributes.
		/// </summary>
		/// <remarks>
		/// Reads an object info(Object meta information like its type and its
		/// values) from the database file
		/// <p/>
		/// <pre>
		/// reads its type and then read all its attributes.
		/// If one attribute is a non native object, it will be read (recursivly).
		/// &lt;p/&gt;
		/// </pre>
		/// </remarks>
		/// <param name="classInfo">
		/// If null, we are probably reading a native instance : String
		/// for example
		/// </param>
		/// <param name="position"></param>
		/// <param name="useCache">
		/// To indicate if cache must be used. If not, the old version of
		/// the object will read
		/// </param>
		/// <returns>The object abstract meta representation @</returns>
		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo ReadObjectInfoFromPosition
			(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo classInfo, long objectPosition, 
			bool useCache, bool returnObjects)
		{
			currentDepth++;
			try
			{
				// Protection against bad parameter value
				if (objectPosition > fsi.GetLength())
				{
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.InstancePositionOutOfFile
						.AddParameter(objectPosition).AddParameter(fsi.GetLength()));
				}
				if (objectPosition == NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
					.DeletedObjectPosition || objectPosition == NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
					.NullObjectPosition)
				{
					// TODO Is this correct ?
					return new NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeDeletedObjectInfo(objectPosition
						, null);
				}
				NeoDatis.Odb.Core.Transaction.ICache cache = storageEngine.GetSession(true).GetCache
					();
				// Read block size and block type
				// block type is used to decide what to do
				fsi.SetReadPosition(objectPosition);
				// Reads the block size
				int blockSize = fsi.ReadInt("object block size");
				// And the block type
				byte blockType = fsi.ReadByte("object block type");
				// Null objects
				if (NeoDatis.Odb.Impl.Core.Layers.Layer3.Block.BlockTypes.IsNullNonNativeObject(blockType
					))
				{
					return new NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeNullObjectInfo(classInfo
						);
				}
				if (NeoDatis.Odb.Impl.Core.Layers.Layer3.Block.BlockTypes.IsNullNativeObject(blockType
					))
				{
					return NeoDatis.Odb.Core.Layers.Layer2.Meta.NullNativeObjectInfo.GetInstance();
				}
				// Deleted objects
				if (NeoDatis.Odb.Impl.Core.Layers.Layer3.Block.BlockTypes.IsDeletedObject(blockType
					))
				{
					return new NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeDeletedObjectInfo(objectPosition
						, null);
				}
				// Checks if what we are reading is only a pointer to the real
				// block, if
				// it is the case, just recall this method with the right position
				if (NeoDatis.Odb.Impl.Core.Layers.Layer3.Block.BlockTypes.IsPointer(blockType))
				{
					throw new NeoDatis.Odb.CorruptedDatabaseException(NeoDatis.Odb.Core.NeoDatisError
						.FoundPointer.AddParameter(objectPosition));
				}
				// Native of non native object ?
				if (NeoDatis.Odb.Impl.Core.Layers.Layer3.Block.BlockTypes.IsNative(blockType))
				{
					// Reads the odb type id of the native objects
					int odbTypeId = fsi.ReadInt();
					// Reads a boolean to know if object is null
					bool isNull = fsi.ReadBoolean("Native object is null ?");
					if (isNull)
					{
						return new NeoDatis.Odb.Core.Layers.Layer2.Meta.NullNativeObjectInfo(odbTypeId);
					}
					// last parameter is false=> no need to read native object
					// header, it has been done
					return ReadNativeObjectInfo(odbTypeId, objectPosition, useCache, returnObjects, false
						);
				}
				if (NeoDatis.Odb.Impl.Core.Layers.Layer3.Block.BlockTypes.IsNonNative(blockType))
				{
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ObjectReaderDirectCall
						);
				}
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.UnknownBlockType
					.AddParameter(blockType).AddParameter(fsi.GetPosition() - 1));
			}
			finally
			{
				currentDepth--;
			}
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo ReadNonNativeObjectInfoFromOid
			(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo classInfo, NeoDatis.Odb.OID oid, 
			bool useCache, bool returnObjects)
		{
			// FIXME if useCache, why not directly search the cache?
			long position = GetObjectPositionFromItsOid(oid, useCache, false);
			if (position == NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.DeletedObjectPosition)
			{
				return new NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeDeletedObjectInfo(position
					, oid);
			}
			if (position == NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.ObjectDoesNotExist)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ObjectWithOidDoesNotExist
					.AddParameter(oid));
			}
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoi = ReadNonNativeObjectInfoFromPosition
				(classInfo, oid, position, useCache, returnObjects);
			// Manage CS select triggers here
			if (!storageEngine.IsLocal())
			{
				// Lazy instantiation
				if (triggerManager == null)
				{
					triggerManager = storageEngine.GetTriggerManager();
				}
				string fullClassName = nnoi.GetClassInfo().GetFullClassName();
				if (triggerManager.HasSelectTriggersFor(fullClassName))
				{
					// We are in cs mode, manage select triggers here
					triggerManager.ManageSelectTriggerAfter(fullClassName, nnoi, oid);
				}
			}
			return nnoi;
		}

		/// <summary>Reads a non non native Object Info (Layer2) from its position</summary>
		/// <param name="classInfo"></param>
		/// <param name="oid">can be null</param>
		/// <param name="position"></param>
		/// <param name="useCache"></param>
		/// <param name="returnInstance"></param>
		/// <returns>The meta representation of the object @</returns>
		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo ReadNonNativeObjectInfoFromPosition
			(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo classInfo, NeoDatis.Odb.OID oid, 
			long position, bool useCache, bool returnInstance)
		{
			NeoDatis.Odb.Core.Transaction.ISession lsession = storageEngine.GetSession(true);
			// Get a temporary cache just to cache NonNativeObjectInfo being read to
			// avoid duplicated reads
			NeoDatis.Odb.Core.Transaction.ICache cache = lsession.GetCache();
			NeoDatis.Odb.Core.Transaction.ITmpCache tmpCache = lsession.GetTmpCache();
			// ICache tmpCache =cache;
			// We are dealing with a non native object
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo objectInfo = null;
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
			{
				NeoDatis.Tool.DLogger.Debug(DepthToSpaces() + "Reading Non Native Object info with oid "
					 + oid);
			}
			// If the object is already being read, then return from the cache
			if (tmpCache.IsReadingObjectInfoWithOid(oid))
			{
				return tmpCache.GetReadingObjectInfoFromOid(oid);
			}
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader objectInfoHeader = GetObjectInfoHeader
				(oid, position, useCache, cache);
			if (classInfo == null)
			{
				classInfo = storageEngine.GetSession(true).GetMetaModel().GetClassInfoFromId(objectInfoHeader
					.GetClassInfoId());
			}
			oid = objectInfoHeader.GetOid();
			// if class info do not match, reload class info
			if (!classInfo.GetId().Equals(objectInfoHeader.GetClassInfoId()))
			{
				classInfo = storageEngine.GetSession(true).GetMetaModel().GetClassInfoFromId(objectInfoHeader
					.GetClassInfoId());
			}
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
			{
				NeoDatis.Tool.DLogger.Debug(DepthToSpaces() + "Reading Non Native Object info of "
					 + (classInfo == null ? "?" : classInfo.GetFullClassName()) + " at " + objectInfoHeader
					.GetPosition() + " with id " + oid);
				NeoDatis.Tool.DLogger.Debug(DepthToSpaces() + "  Object Header is " + objectInfoHeader
					);
			}
			objectInfo = new NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo(objectInfoHeader
				, classInfo);
			objectInfo.SetOid(oid);
			objectInfo.SetClassInfo(classInfo);
			objectInfo.SetPosition(objectInfoHeader.GetPosition());
			// Adds the Object Info in cache. The remove (cache clearing) is done by
			// the Query Executor. This tmp cache is used to resolve cyclic reference problem.
			// When an object has cyclic reference, if we don t cache the object info, we will read the reference for ever!
			// With the cache , we detect the cyclic reference and return what has been read already
			tmpCache.StartReadingObjectInfoWithOid(objectInfo.GetOid(), objectInfo);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassAttributeInfo cai = null;
			NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo aoi = null;
			long attributeIdentification = -1;
			NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.PendingReading
				> pendingReadings = new NeoDatis.Tool.Wrappers.List.OdbArrayList<NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.PendingReading
				>();
			for (int id = 1; id <= classInfo.GetMaxAttributeId(); id++)
			{
				cai = objectInfo.GetClassInfo().GetAttributeInfoFromId(id);
				if (cai == null)
				{
					// the attribute does not exist anymore
					continue;
				}
				attributeIdentification = objectInfoHeader.GetAttributeIdentificationFromId(id);
				if (attributeIdentification == NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
					.NullObjectPosition || attributeIdentification == NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
					.NullObjectIdId)
				{
					if (cai.IsNative())
					{
						aoi = NeoDatis.Odb.Core.Layers.Layer2.Meta.NullNativeObjectInfo.GetInstance();
					}
					else
					{
						aoi = new NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeNullObjectInfo();
					}
					objectInfo.SetAttributeValue(id, aoi);
				}
				else
				{
					// Here we can not use cai.isNonNative because of interfaces :
					// because an interface will always be considered as non native
					// (Object for example) but
					// could contain a String for example. So we assume that if
					// attributeIdentification is negative
					// the object is non native,if positive the object is native.
					if (attributeIdentification < 0)
					{
						// ClassInfo ci =
						// storageEngine.getSession(true).getMetaModel().getClassInfo(cai.getFullClassname(),
						// true);
						// For non native objects. attribute identification is the
						// oid (*-1)
						NeoDatis.Odb.OID attributeOid = NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(-
							attributeIdentification);
						// We do not read now, store the reading as pending and
						// reads it later
						pendingReadings.Add(new NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.PendingReading
							(id, null, attributeOid));
					}
					else
					{
						aoi = ReadObjectInfo(attributeIdentification, useCache, returnInstance);
						objectInfo.SetAttributeValue(id, aoi);
					}
				}
			}
			NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.PendingReading pr = null;
			for (int i = 0; i < pendingReadings.Count; i++)
			{
				pr = pendingReadings[i];
				// If object is not in connected zone , the cache must be used
				bool useCacheForAttribute = useCache || !cache.ObjectWithIdIsInCommitedZone(pr.GetAttributeOID
					());
				aoi = ReadNonNativeObjectInfoFromOid(pr.GetCi(), pr.GetAttributeOID(), useCacheForAttribute
					, returnInstance);
				objectInfo.SetAttributeValue(pr.GetId(), aoi);
			}
			// FIXME Check if instance is being built on client server mode
			if (returnInstance)
			{
			}
			// objectInfo.setObject(instanceBuilder.buildOneInstance(objectInfo));
			return objectInfo;
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap ReadObjectInfoValuesFromOID
			(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo classInfo, NeoDatis.Odb.OID oid, 
			bool useCache, NeoDatis.Tool.Wrappers.List.IOdbList<string> attributeNames, NeoDatis.Tool.Wrappers.List.IOdbList
			<string> relationAttributeNames, int recursionLevel, string[] orderByFields)
		{
			long position = GetObjectPositionFromItsOid(oid, useCache, true);
			return ReadObjectInfoValuesFromPosition(classInfo, oid, position, useCache, attributeNames
				, relationAttributeNames, recursionLevel, orderByFields);
		}

		/// <param name="classInfo">The class info of the objects to be returned</param>
		/// <param name="oid">The Object id of the object to return data</param>
		/// <param name="position">The position of the object to read</param>
		/// <param name="useCache">To indicate if cache must be used</param>
		/// <param name="attributeNames">
		/// The list of the attribute name for which we need to return a
		/// value, an attributename can contain relation like profile.name
		/// </param>
		/// <param name="relationAttributeNames">
		/// The original names of attributes to read the values, an
		/// attributename can contain relation like profile.name
		/// </param>
		/// <param name="recursionLevel">The recursion level of this call</param>
		/// <param name="orderByFields">?</param>
		/// <returns>
		/// A Map where keys are attributes names and values are the values
		/// of there attributes @
		/// </returns>
		protected virtual AttributeValuesMap ReadObjectInfoValuesFromPosition(ClassInfo classInfo, OID oid, long position, bool useCache, IOdbList<string> attributeNames, IOdbList<string> relationAttributeNames, int recursionLevel
			, string[] orderByFields)
		{
			currentDepth++;
			// The resulting map
			AttributeValuesMap map = new AttributeValuesMap();
			// Protection against bad parameter value
			if (position > fsi.GetLength())
			{
				throw new ODBRuntimeException(NeoDatisError.InstancePositionOutOfFile.AddParameter(position).AddParameter(fsi.GetLength()));
			}
			ICache cache = storageEngine.GetSession(true).GetCache
				();
			// If object is already being read, simply return its cache - to avoid
			// stackOverflow for cyclic references
			// FIXME check this : should we use cache?
			// Go to the object position
			fsi.SetReadPosition(position);
			// Read the block size of the object
			int blockSize = fsi.ReadInt();
			// Read the block type of the object
			byte blockType = fsi.ReadByte();
			if (BlockTypes.IsNull(blockType) || BlockTypes.IsDeletedObject(blockType))
			{
				return map;
			}
			// Checks if what we are reading is only a pointer to the real block, if
			// it is the case, Throw an exception. Pointer are not used anymore
			if (BlockTypes.IsPointer(blockType))	{
				throw new CorruptedDatabaseException(NeoDatisError.FoundPointer.AddParameter(oid).AddParameter(position));
			}
			try
			{
				// Read the header of the object, no need to cache when reading
				// object infos
				// For local mode, we need to use cache to get unconnected objects.
				// TestDelete.test14
				ObjectInfoHeader objectInfoHeader = GetObjectInfoHeader
					(oid, position, true, cache);
				// Get the object id
				oid = objectInfoHeader.GetOid();
				// If class info is not defined, define it
				if (classInfo == null)
				{
					classInfo = storageEngine.GetSession(true).GetMetaModel().GetClassInfoFromId(objectInfoHeader.GetClassInfoId());
				}
				if (recursionLevel == 0)
				{
					map.SetObjectInfoHeader(objectInfoHeader);
				}
				// If object is native, it can have attributes, just return the
				// empty
				// map
				if (NeoDatis.Odb.Impl.Core.Layers.Layer3.Block.BlockTypes.IsNative(blockType))
				{
					return map;
				}
				ClassAttributeInfo cai = null;
				int nbAttributes = attributeNames.Count;
				string attributeNameToSearch = null;
				string relationNameToSearch = null;
				string singleAttributeName = null;
				bool mustNavigate = false;
				// The query contains a list of attribute to search
				// Loop on attribute to search
				for (int attributeIndex = 0; attributeIndex < nbAttributes; attributeIndex++)
				{
					attributeNameToSearch = attributeNames[attributeIndex];
					relationNameToSearch = (string)relationAttributeNames[attributeIndex];
					// If an attribute name has a ., it is a relation
					mustNavigate = attributeNameToSearch.IndexOf(".") != -1;
					long attributeIdentification = -1;
					long attributePosition = -1;
					NeoDatis.Odb.OID attributeOid = null;
					if (mustNavigate)
					{
						// Get the relation name and the relation attribute name
						// profile.name => profile = singleAttributeName, name =
						// relationAttributeName
						int firstDotIndex = attributeNameToSearch.IndexOf(".");
						string relationAttributeName = OdbString.Substring(attributeNameToSearch, firstDotIndex + 1);
						singleAttributeName = OdbString.Substring(attributeNameToSearch	, 0, firstDotIndex);
						int attributeId = classInfo.GetAttributeId(singleAttributeName);
						if (attributeId == -1)
						{
							throw new ODBRuntimeException(NeoDatisError.CriteriaQueryUnknownAttribute.AddParameter(attributeNameToSearch).AddParameter(classInfo.GetFullClassName()));
						}
						cai = classInfo.GetAttributeInfoFromId(attributeId);
						// Gets the identification (id or position from the object
						// info) for the attribute with the id of the class
						// attribute info
						attributeIdentification = objectInfoHeader.GetAttributeIdentificationFromId(cai.GetId());
						// When object is non native, then attribute identification
						// is the oid of the object. It is stored as negative, so we
						// must do *-1
						if (!cai.IsNative())
						{
							// Relations can be null
							if (attributeIdentification == StorageEngineConstant.NullObjectIdId)
							{
								map.Add(attributeNameToSearch, null);
								continue;
							}
							attributeOid = OIDFactory.BuildObjectOID(-attributeIdentification);
							attributePosition = GetObjectPositionFromItsOid(attributeOid, useCache, false);
							IOdbList<string> list1 = new OdbArrayList<string>(1);
							list1.Add(relationAttributeName);
							IOdbList<string> list2 = new OdbArrayList<string>(1);
							list2.Add(attributeNameToSearch);
							map.PutAll(ReadObjectInfoValuesFromPosition(cai.GetClassInfo(), attributeOid, attributePosition	, useCache, list1, list2, recursionLevel + 1, orderByFields));
						}
						else
						{
							throw new ODBRuntimeException(NeoDatisError.CriteriaQueryUnknownAttribute.AddParameter(attributeNameToSearch).AddParameter(classInfo.GetFullClassName()));
						}
					}
					else
					{
						int attributeId = classInfo.GetAttributeId(attributeNameToSearch);
						if (attributeId == -1)
						{
							throw new ODBRuntimeException(NeoDatisError.CriteriaQueryUnknownAttribute.AddParameter(attributeNameToSearch).AddParameter(classInfo.GetFullClassName()));
						}
						cai = classInfo.GetAttributeInfoFromId(attributeId);
						// Gets the identification (id or position from the object
						// info) for the attribute with the id of the class
						// attribute info
						attributeIdentification = objectInfoHeader.GetAttributeIdentificationFromId(cai.GetId
							());
						// When object is non native, then attribute identification
						// is the oid of the object. It is stored as negative, so we
						// must do *-1
						if (cai.IsNonNative())
						{
							attributeOid = NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(-attributeIdentification);
						}
						// For non native object, the identification is the oid,
						// which is stored as negative long
						// @TODO The attributeIdentification <0 clause should not be
						// necessary
						// But there is a case (found by Jeremias) where even for
						// non
						// native the attribute
						// is a position and not an id! identification
						if (cai.IsNonNative() && attributeIdentification < 0)
						{
							attributePosition = GetObjectPositionFromItsOid(attributeOid, useCache, false);
						}
						else
						{
							attributePosition = attributeIdentification;
						}
						if (attributePosition == NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
							.DeletedObjectPosition || attributePosition == NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
							.NullObjectPosition || attributePosition == NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
							.FieldDoesNotExist)
						{
							// TODO is this correct?
							continue;
						}
						fsi.SetReadPosition(attributePosition);
						NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo aoi = null;
						object @object = null;
						if (cai.IsNative())
						{
							aoi = ReadNativeObjectInfo(cai.GetAttributeType().GetId(), attributePosition, useCache
								, true, true);
							@object = aoi.GetObject();
							map.Add(relationNameToSearch, @object);
						}
						else
						{
							NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoi = ReadNonNativeObjectInfoFromOid
								(cai.GetClassInfo(), attributeOid, true, false);
							@object = nnoi.GetObject();
							if (@object == null)
							{
							}
							//object = instanceBuilder.buildOneInstance(nnoi);
							map.Add(relationNameToSearch, nnoi.GetOid());
						}
					}
				}
				return map;
			}
			finally
			{
				currentDepth--;
			}
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader GetObjectInfoHeader
			(NeoDatis.Odb.OID oid, long position, bool useCache, NeoDatis.Odb.Core.Transaction.ICache
			 cache)
		{
			// first check if the object info pointers exist in the cache
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader objectInfoHeader = null;
			if (useCache && oid != null)
			{
				objectInfoHeader = cache.GetObjectInfoHeaderFromOid(oid, false);
			}
			if (objectInfoHeader == null)
			{
				// Here we read by position because it is possible to have the
				// oid == null. And it is faster by position than by oid
				objectInfoHeader = ReadObjectInfoHeaderFromPosition(oid, position, false);
				bool oidWasNull = oid == null;
				oid = objectInfoHeader.GetOid();
				if (useCache)
				{
					bool needToUpdateCache = true;
					if (oidWasNull)
					{
						// The oid was null, now we have it, check the cache again !
						NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader cachedOih = cache.GetObjectInfoHeaderFromOid
							(oid, false);
						if (cachedOih != null)
						{
							// Then use the one from the cache
							objectInfoHeader = cachedOih;
							// In this case the cache is up to date , no need to
							// update
							needToUpdateCache = false;
						}
					}
					if (needToUpdateCache)
					{
						cache.AddObjectInfo(objectInfoHeader);
					}
				}
			}
			return objectInfoHeader;
		}

		/// <summary>
		/// Read the header of a native attribute
		/// <pre>
		/// The header contains
		/// - The block size = int
		/// - The block type = byte
		/// - The OdbType ID = int
		/// - A boolean to indicate if object is nulls.
		/// </summary>
		/// <remarks>
		/// Read the header of a native attribute
		/// <pre>
		/// The header contains
		/// - The block size = int
		/// - The block type = byte
		/// - The OdbType ID = int
		/// - A boolean to indicate if object is nulls.
		/// This method reads all the bytes and then convert the byte array to the values
		/// </pre>
		/// </remarks>
		/// <param name="odbTypeId"></param>
		/// <param name="isNull"></param>
		/// <param name="writeDataInTransaction"></param>
		/// <></>
		protected virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.NativeAttributeHeader ReadNativeAttributeHeader
			()
		{
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NativeAttributeHeader nah = new NeoDatis.Odb.Core.Layers.Layer2.Meta.NativeAttributeHeader
				();
			int size = NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Integer.GetSize() + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
				.Byte.GetSize() + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Integer.GetSize()
				 + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Boolean.GetSize();
			byte[] bytes = fsi.ReadBytes(size);
			int blockSize = byteArrayConverter.ByteArrayToInt(bytes, 0);
			byte blockType = bytes[4];
			int odbTypeId = byteArrayConverter.ByteArrayToInt(bytes, 5);
			bool isNull = byteArrayConverter.ByteArrayToBoolean(bytes, 9);
			nah.SetBlockSize(blockSize);
			nah.SetBlockType(blockType);
			nah.SetOdbTypeId(odbTypeId);
			nah.SetNull(isNull);
			return nah;
		}

		/// <summary>Reads a meta representation of a native object</summary>
		/// <param name="odbDeclaredTypeId">
		/// The type of attribute declared in the ClassInfo. May be
		/// different from actual attribute type in caso of OID and
		/// OdbObjectId
		/// </param>
		/// <param name="position"></param>
		/// <param name="useCache"></param>
		/// <param name="returnObject"></param>
		/// <param name="readHeader"></param>
		/// <returns>The native object representation @</returns>
		private NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo ReadNativeObjectInfo
			(int odbDeclaredTypeId, long position, bool useCache, bool returnObject, bool readHeader
			)
		{
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
			{
				NeoDatis.Tool.DLogger.Debug(DepthToSpaces() + "Reading native object of type " + 
					NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.GetNameFromId(odbDeclaredTypeId) + 
					" at position " + position);
			}
			// The realType is initialized with the declared type
			int realTypeId = odbDeclaredTypeId;
			if (readHeader)
			{
				NeoDatis.Odb.Core.Layers.Layer2.Meta.NativeAttributeHeader nah = ReadNativeAttributeHeader
					();
				// since version 3 of ODB File Format, the native object header has
				// an info to indicate
				// if object is null!
				if (nah.IsNull())
				{
					return new NeoDatis.Odb.Core.Layers.Layer2.Meta.NullNativeObjectInfo(odbDeclaredTypeId
						);
				}
				realTypeId = nah.GetOdbTypeId();
			}
			if (NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.IsAtomicNative(realTypeId))
			{
				return ReadAtomicNativeObjectInfo(position, realTypeId);
			}
			if (NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.IsNull(realTypeId))
			{
				return new NeoDatis.Odb.Core.Layers.Layer2.Meta.NullNativeObjectInfo(realTypeId);
			}
			if (NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.IsCollection(realTypeId))
			{
				return ReadCollection(position, useCache, returnObject);
			}
			if (NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.IsArray(realTypeId))
			{
				return ReadArray(position, useCache, returnObject);
			}
			if (NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.IsMap(realTypeId))
			{
				return ReadMap(position, useCache, returnObject);
			}
			if (NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.IsEnum(realTypeId))
			{
				return ReadEnumObjectInfo(position, realTypeId);
			}
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.NativeTypeNotSupported
				.AddParameter(realTypeId));
		}

		public virtual object ReadAtomicNativeObjectInfoAsObject(long position, int odbTypeId
			)
		{
			object o = null;
			switch (odbTypeId)
			{
				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.ByteId:
				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeByteId:
				{
					o = fsi.ReadByte("atomic");
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.BooleanId:
				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeBooleanId:
				{
					bool b = fsi.ReadBoolean("atomic");
					if (b)
					{
						o = true;
					}
					else
					{
						o = false;
					}
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.CharacterId:
				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeCharId:
				{
					o = fsi.ReadChar("atomic");
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.FloatId:
				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeFloatId:
				{
					o = fsi.ReadFloat("atomic");
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.DoubleId:
				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeDoubleId:
				{
					o = fsi.ReadDouble("atomic");
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.IntegerId:
				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeIntId:
				{
					o = fsi.ReadInt("atomic");
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.LongId:
				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeLongId:
				{
					o = fsi.ReadLong("atomic");
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.ShortId:
				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeShortId:
				{
					o = fsi.ReadShort("atomic");
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.BigDecimalId:
				{
					o = fsi.ReadBigDecimal("atomic");
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.BigIntegerId:
				{
					o = fsi.ReadBigInteger("atomic");
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.DateId:
				{
					o = fsi.ReadDate("atomic");
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.DateSqlId:
				{
					o = fsi.ReadDate("atomic");
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.DateTimestampId:
				{
					o = fsi.ReadDate("atomic");
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.ObjectOidId:
				{
					long oid = fsi.ReadLong("oid");
					o = NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(oid);
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.ClassOidId:
				{
					long cid = fsi.ReadLong("oid");
					o = NeoDatis.Odb.Core.Oid.OIDFactory.BuildClassOID(cid);
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.StringId:
				{
					o = fsi.ReadString(true);
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.EnumId:
				{
					o = fsi.ReadString(false);
					break;
				}
			}
			if (o == null)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.NativeTypeNotSupported
					.AddParameter(odbTypeId).AddParameter(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
					.GetNameFromId(odbTypeId)));
			}
			return o;
		}

		/// <summary>Reads an atomic object</summary>
		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo ReadAtomicNativeObjectInfo
			(long position, int odbTypeId)
		{
			object @object = ReadAtomicNativeObjectInfoAsObject(position, odbTypeId);
			return new NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo(@object, odbTypeId
				);
		}

		/// <summary>Reads an enum object</summary>
		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.EnumNativeObjectInfo ReadEnumObjectInfo
			(long position, int odbTypeId)
		{
			long enumClassInfoId = fsi.ReadLong("EnumClassInfoId");
			string enumValue = fsi.ReadString(true);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo enumCi = GetSession().GetMetaModel
				().GetClassInfoFromId(NeoDatis.Odb.Core.Oid.OIDFactory.BuildClassOID(enumClassInfoId
				));
			return new NeoDatis.Odb.Core.Layers.Layer2.Meta.EnumNativeObjectInfo(enumCi, enumValue
				);
		}

		/// <summary>
		/// Reads a collection from the database file
		/// <p/>
		/// <pre>
		/// This method do not returns the object but a collection of representation of the objects using AsbtractObjectInfo
		/// &lt;p/&gt;
		/// The conversion to a real Map object will be done by the buildInstance method
		/// </pre>
		/// </summary>
		/// <param name="position">The position to be read</param>
		/// <returns>The meta representation of a collection @</returns>
		/// <exception cref="System.MemberAccessException">System.MemberAccessException</exception>
		/// <exception cref="Java.Lang.InstantiationException">Java.Lang.InstantiationException
		/// 	</exception>
		private NeoDatis.Odb.Core.Layers.Layer2.Meta.CollectionObjectInfo ReadCollection(
			long position, bool useCache, bool returnObjects)
		{
			long[] objectIdentifications;
			NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo aoi = null;
			string realCollectionClassName = fsi.ReadString(false, "Real collection class name"
				);
			// read the size of the collection
			int collectionSize = fsi.ReadInt("Collection size");
			System.Type clazz = null;
			System.Collections.Generic.ICollection<NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
				> c = new System.Collections.Generic.List<NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
				>(collectionSize);
			// build a n array to store all element positions
			objectIdentifications = new long[collectionSize];
			for (int i = 0; i < collectionSize; i++)
			{
				objectIdentifications[i] = fsi.ReadLong("position of element " + (i + 1));
			}
			for (int i = 0; i < collectionSize; i++)
			{
				try
				{
					aoi = ReadObjectInfo(objectIdentifications[i], useCache, returnObjects);
					if (!(aoi is NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeDeletedObjectInfo))
					{
						c.Add(aoi);
					}
				}
				catch (System.Exception e)
				{
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.InternalError
						.AddParameter("in ObjectReader.readCollection - at position " + position), e);
				}
			}
			NeoDatis.Odb.Core.Layers.Layer2.Meta.CollectionObjectInfo coi = new NeoDatis.Odb.Core.Layers.Layer2.Meta.CollectionObjectInfo
				(c);
			coi.SetRealCollectionClassName(realCollectionClassName);
			return coi;
		}

		/// <summary>Reads an array from the database file</summary>
		/// <param name="position">The position to be read</param>
		/// <returns>The Collection or the array @</returns>
		private NeoDatis.Odb.Core.Layers.Layer2.Meta.ArrayObjectInfo ReadArray(long position
			, bool useCache, bool returnObjects)
		{
			long[] objectIdentifications = null;
			bool arrayComponentHasFixedSize = true;
			string realArrayComponentClassName = fsi.ReadString(false, "real array class name"
				);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType subTypeId = NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
				.GetFromName(realArrayComponentClassName);
			bool componentIsNative = subTypeId.IsNative();
			// read the size of the array
			int arraySize = fsi.ReadInt();
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
			{
				NeoDatis.Tool.DLogger.Debug(DepthToSpaces() + "reading an array of " + realArrayComponentClassName
					 + " with " + arraySize + " elements");
			}
			// Class clazz = ODBClassPool.getClass(realArrayClassName);
			// Object array = Array.newInstance(clazz, arraySize);
			object[] array = new object[arraySize];
			// build a n array to store all element positions
			objectIdentifications = new long[arraySize];
			for (int i = 0; i < arraySize; i++)
			{
				objectIdentifications[i] = fsi.ReadLong();
			}
			for (int i = 0; i < arraySize; i++)
			{
				try
				{
					if (objectIdentifications[i] != NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
						.NullObjectIdId)
					{
						object o = ReadObjectInfo(objectIdentifications[i], useCache, returnObjects);
						if (!(o is NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeDeletedObjectInfo))
						{
							NeoDatis.Tool.Wrappers.OdbArray.SetValue(array, i, o);
						}
					}
					else
					{
						if (componentIsNative)
						{
							NeoDatis.Tool.Wrappers.OdbArray.SetValue(array, i, NeoDatis.Odb.Core.Layers.Layer2.Meta.NullNativeObjectInfo
								.GetInstance());
						}
						else
						{
							NeoDatis.Tool.Wrappers.OdbArray.SetValue(array, i, new NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeNullObjectInfo
								());
						}
					}
				}
				catch (System.Exception e)
				{
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.InternalError
						.AddParameter("in ObjectReader.readArray - at position " + position), e);
				}
			}
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ArrayObjectInfo aoi = new NeoDatis.Odb.Core.Layers.Layer2.Meta.ArrayObjectInfo
				(array);
			aoi.SetRealArrayComponentClassName(realArrayComponentClassName);
			aoi.SetComponentTypeId(subTypeId.GetId());
			return aoi;
		}

		/// <summary>
		/// Reads a map from the database file
		/// <p/>
		/// <pre>
		/// WARNING : this method returns a collection representation of the map
		/// &lt;p/&gt;
		/// Firts it does not return the objects but its meta information using AbstractObjectInfo
		/// &lt;p/&gt;
		/// So for example, the map [1=olivier,2=chico]
		/// will be returns as a collection : [1,olivier,2,chico]
		/// and each element of the collection is an abstractObjectInfo (NativeObjectInfo or NonNativeObjectInfo)
		/// &lt;p/&gt;
		/// The conversion to a real Map object will be done by the buildInstance method
		/// </pre>
		/// </summary>
		/// <param name="position">The position to be read</param>
		/// <param name="useCache"></param>
		/// <param name="returnObjects"></param>
		/// <returns>The Map @</returns>
		/// <exception cref="System.MemberAccessException">System.MemberAccessException</exception>
		/// <exception cref="Java.Lang.InstantiationException">Java.Lang.InstantiationException
		/// 	</exception>
		private MapObjectInfo ReadMap(long position, bool useCache, bool returnObjects)
		{
			long[] objectIdentifications;
			AbstractObjectInfo aoiKey = null;
			AbstractObjectInfo aoiValue = null;
			// Reads the real map class
			string realMapClassName = fsi.ReadString(false);
			// read the size of the map
			int mapSize = fsi.ReadInt();
			IDictionary<AbstractObjectInfo,AbstractObjectInfo> map = new OdbHashMap<AbstractObjectInfo,AbstractObjectInfo>();
			
			// build a n array to store all element positions
			objectIdentifications = new long[mapSize * 2];
			for (int i = 0; i < mapSize * 2; i++)
			{
				objectIdentifications[i] = fsi.ReadLong();
			}
			for (int i = 0; i < mapSize; i++)
			{
				try
				{
					aoiKey = ReadObjectInfo(objectIdentifications[2 * i], useCache, returnObjects);
					aoiValue = ReadObjectInfo(objectIdentifications[2 * i + 1], useCache, returnObjects
						);
					if (!aoiKey.IsDeletedObject() && !aoiValue.IsDeletedObject())
					{
						map.Add(aoiKey, aoiValue);
					}
				}
				catch (System.Exception e)
				{
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.InternalError
						.AddParameter("in ObjectReader.readMap - at position " + position), e);
				}
			}
			return new MapObjectInfo(map, realMapClassName);
		}

		/// <summary>Gets the next object oid of the object with the specific oid</summary>
		/// <param name="position"></param>
		/// <returns>
		/// The position of the next object. If there is no next object,
		/// return -1 @
		/// </returns>
		public virtual NeoDatis.Odb.OID GetNextObjectOID(NeoDatis.Odb.OID oid)
		{
			long position = storageEngine.GetObjectWriter().GetIdManager().GetObjectPositionWithOid
				(oid, true);
			fsi.SetReadPosition(position + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.ObjectOffsetNextObjectOid);
			return NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(fsi.ReadLong());
		}

		public virtual long ReadOidPosition(NeoDatis.Odb.OID oid)
		{
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
			{
				NeoDatis.Tool.DLogger.Debug("  Start of readOidPosition for oid " + oid);
			}
			long blockNumber = GetIdBlockNumberOfOid(oid);
			long blockPosition = -1;
			blockPosition = GetIdBlockPositionFromNumber(blockNumber);
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
			{
				NeoDatis.Tool.DLogger.Debug("  Block number of oid " + oid + " is " + blockNumber
					 + " / block position = " + blockPosition);
			}
			long position = blockPosition + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.BlockIdOffsetForStartOfRepetition + ((oid.GetObjectId() - 1) % NeoDatis.Odb.OdbConfiguration
				.GetNB_IDS_PER_BLOCK()) * NeoDatis.Odb.OdbConfiguration.GetID_BLOCK_REPETITION_SIZE
				();
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
			{
				NeoDatis.Tool.DLogger.Debug("  End of readOidPosition for oid " + oid + " returning position "
					 + position);
			}
			return position;
		}

		public virtual object GetObjectFromOid(NeoDatis.Odb.OID oid, bool returnInstance, 
			bool useCache)
		{
			long position = GetObjectPositionFromItsOid(oid, useCache, true);
			object o = ReadNonNativeObjectAtPosition(position, useCache, returnInstance);
			// Clear the tmp cache. This cache is use to resolve cyclic references
			GetSession().GetTmpCache().ClearObjectInfos();
			return o;
		}

		/// <summary>Returns the name of the class of an object from its position</summary>
		/// <param name="objectPosition"></param>
		/// <returns>The object class name @</returns>
		public virtual string GetObjectTypeFromPosition(long objectPosition)
		{
			long blockPosition = objectPosition + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.ObjectOffsetBlockType;
			fsi.SetReadPosition(blockPosition);
			byte blockType = fsi.ReadByte();
			if (NeoDatis.Odb.Impl.Core.Layers.Layer3.Block.BlockTypes.IsNull(blockType))
			{
				NeoDatis.Odb.OID classIdForNullObject = NeoDatis.Odb.Core.Oid.OIDFactory.BuildClassOID
					(fsi.ReadLong("class id of object"));
				return "null " + storageEngine.GetSession(true).GetMetaModel().GetClassInfoFromId
					(classIdForNullObject).GetFullClassName();
			}
			long classIdPosition = objectPosition + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.ObjectOffsetClassInfoId;
			fsi.SetReadPosition(classIdPosition);
			NeoDatis.Odb.OID classId = NeoDatis.Odb.Core.Oid.OIDFactory.BuildClassOID(fsi.ReadLong
				("class id of object"));
			return storageEngine.GetSession(true).GetMetaModel().GetClassInfoFromId(classId).
				GetFullClassName();
		}

		/// <summary>Gets the real object position from its OID</summary>
		/// <param name="oid">The oid of the object to get the position</param>
		/// <param name="throwException">
		/// To indicate if an exception must be thrown if object is not
		/// found
		/// </param>
		/// <returns>
		/// The object position, if object has been marked as deleted then
		/// return StorageEngineConstant.DELETED_OBJECT_POSITION @
		/// </returns>
		public virtual long GetObjectPositionFromItsOid(NeoDatis.Odb.OID oid, bool useCache
			, bool throwException)
		{
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
			{
				NeoDatis.Tool.DLogger.Debug("  getObjectPositionFromItsId for oid " + oid);
			}
			// Check if oid is in cache
			long position = NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.ObjectIsNotInCache;
			if (useCache)
			{
				// This return -1 if not in the cache
				position = storageEngine.GetSession(true).GetCache().GetObjectPositionByOid(oid);
			}
			// FIXME Check if we need this. Removing it causes the TestDelete.test6 to fail 
			if (position == NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.DeletedObjectPosition)
			{
				if (throwException)
				{
					throw new NeoDatis.Odb.CorruptedDatabaseException(NeoDatis.Odb.Core.NeoDatisError
						.ObjectIsMarkedAsDeletedForOid.AddParameter(oid));
				}
				return NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant.DeletedObjectPosition;
			}
			if (position != NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.ObjectIsNotInCache && position != NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.DeletedObjectPosition)
			{
				return position;
			}
			// The position was not found is the cache
			position = ReadOidPosition(oid);
			position += NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant.BlockIdRepetitionIdStatus;
			fsi.SetReadPosition(position);
			byte idStatus = fsi.ReadByte();
			long objectPosition = fsi.ReadLong();
			if (!NeoDatis.Odb.Impl.Core.Layers.Layer3.Oid.IDStatus.IsActive(idStatus))
			{
				// if object position == 0, The object dos not exist
				if (throwException)
				{
					if (objectPosition == 0)
					{
						throw new NeoDatis.Odb.CorruptedDatabaseException(NeoDatis.Odb.Core.NeoDatisError
							.ObjectWithOidDoesNotExist.AddParameter(oid));
					}
					throw new NeoDatis.Odb.CorruptedDatabaseException(NeoDatis.Odb.Core.NeoDatisError
						.ObjectIsMarkedAsDeletedForOid.AddParameter(oid));
				}
				if (objectPosition == 0)
				{
					return NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant.ObjectDoesNotExist;
				}
				return NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant.DeletedObjectPosition;
			}
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
			{
				NeoDatis.Tool.DLogger.Debug("  object position of object with oid " + oid + " is "
					 + objectPosition);
			}
			return objectPosition;
		}

		/// <param name="blockNumberToFind"></param>
		/// <returns>The block position @</returns>
		private long GetIdBlockPositionFromNumber(long blockNumberToFind)
		{
			//TODO remove new Long
			// first check if it exist in cache
            long lposition = 0;
            
            blockPositions.TryGetValue(blockNumberToFind, out lposition) ;
			if (lposition != 0)
			{
				return lposition;
			}
			long nextBlockPosition = 0;
			long currentBlockPosition = NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.DatabaseHeaderFirstIdBlockPosition;
			int blockNumber = -1;
			while (currentBlockPosition != -1)
			{
				// Gets the next block position
				fsi.SetReadPosition(currentBlockPosition + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
					.BlockIdOffsetForNextBlock);
				nextBlockPosition = fsi.ReadLong();
				// Reads the block number
				blockNumber = fsi.ReadInt();
				if (blockNumber == blockNumberToFind)
				{
					// Put result in map
					blockPositions.Add(blockNumberToFind, currentBlockPosition);
					return currentBlockPosition;
				}
				currentBlockPosition = nextBlockPosition;
			}
			throw new NeoDatis.Odb.CorruptedDatabaseException(NeoDatis.Odb.Core.NeoDatisError
				.BlockNumberDoesExist.AddParameter(blockNumberToFind));
		}

		private long GetIdBlockNumberOfOid(NeoDatis.Odb.OID oid)
		{
			long number = -1;
			long objectId = oid.GetObjectId();
			if (objectId % NeoDatis.Odb.OdbConfiguration.GetNB_IDS_PER_BLOCK() == 0)
			{
				number = objectId / NeoDatis.Odb.OdbConfiguration.GetNB_IDS_PER_BLOCK();
			}
			else
			{
				number = objectId / NeoDatis.Odb.OdbConfiguration.GetNB_IDS_PER_BLOCK() + 1;
			}
			return number;
		}

		/// <summary>Returns information about all OIDs of the database</summary>
		/// <param name="idType"></param>
		/// <returns>@</returns>
		public virtual System.Collections.Generic.IList<long> GetAllIds(byte idType)
		{
			long blockMaxId = 0;
			long currentId = 0;
			byte idTypeRead = 0;
			byte idStatus = 0;
			long nextRepetitionPosition = 0;
			System.Collections.Generic.IList<long> ids = new System.Collections.Generic.List<
				long>(5000);
			long nextBlockPosition = 0;
			long currentBlockPosition = NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.DatabaseHeaderFirstIdBlockPosition;
			while (currentBlockPosition != -1)
			{
				// Gets the next block position
				fsi.SetReadPosition(currentBlockPosition + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
					.BlockIdOffsetForNextBlock);
				nextBlockPosition = fsi.ReadLong();
				// Gets the block max id
				fsi.SetReadPosition(currentBlockPosition + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
					.BlockIdOffsetForMaxId);
				blockMaxId = fsi.ReadLong();
				do
				{
					nextRepetitionPosition = fsi.GetPosition() + NeoDatis.Odb.OdbConfiguration.GetID_BLOCK_REPETITION_SIZE
						();
					idTypeRead = fsi.ReadByte();
					currentId = fsi.ReadLong();
					idStatus = fsi.ReadByte();
					if (idType == idTypeRead && NeoDatis.Odb.Impl.Core.Layers.Layer3.Oid.IDStatus.IsActive
						(idStatus))
					{
						ids.Add(currentId);
					}
					fsi.SetReadPosition(nextRepetitionPosition);
				}
				while (currentId != blockMaxId);
				currentBlockPosition = nextBlockPosition;
			}
			return ids;
		}

		public virtual System.Collections.Generic.IList<NeoDatis.Odb.Impl.Core.Layers.Layer3.Oid.FullIDInfo
			> GetAllIdInfos(string objectTypeToDisplay, byte idType, bool displayObject)
		{
			long blockId = 0;
			long blockMaxId = 0;
			long currentId = 0;
			byte idTypeRead = 0;
			byte idStatus = 0;
			long objectPosition = 0;
			long nextRepetitionPosition = 0;
			string objectType = null;
			System.Collections.Generic.IList<NeoDatis.Odb.Impl.Core.Layers.Layer3.Oid.FullIDInfo
				> idInfos = new System.Collections.Generic.List<NeoDatis.Odb.Impl.Core.Layers.Layer3.Oid.FullIDInfo
				>(5000);
			long nextBlockPosition = 0;
			NeoDatis.Odb.OID prevObjectOID = null;
			NeoDatis.Odb.OID nextObjectOID = null;
			long currentBlockPosition = NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.DatabaseHeaderFirstIdBlockPosition;
			NeoDatis.Odb.Impl.Core.Layers.Layer3.Oid.FullIDInfo info = null;
			string objectToString = "empty";
			while (currentBlockPosition != -1)
			{
				NeoDatis.Tool.DLogger.Debug("Current block position = " + currentBlockPosition);
				fsi.SetReadPosition(currentBlockPosition + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
					.BlockIdOffsetForBlockNumber);
				fsi.SetReadPosition(currentBlockPosition + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
					.BlockIdOffsetForNextBlock);
				nextBlockPosition = fsi.ReadLong();
				// Gets block number
				blockId = fsi.ReadInt();
				blockMaxId = fsi.ReadLong();
				do
				{
					nextRepetitionPosition = fsi.GetPosition() + NeoDatis.Odb.OdbConfiguration.GetID_BLOCK_REPETITION_SIZE
						();
					idTypeRead = fsi.ReadByte();
					currentId = fsi.ReadLong();
					idStatus = fsi.ReadByte();
					objectPosition = fsi.ReadLong();
					if (idType == idTypeRead)
					{
						// && IDStatus.isActive(idStatus)) {
						long currentPosition = fsi.GetPosition();
						if (displayObject)
						{
							NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo aoi = null;
							try
							{
								aoi = ReadNonNativeObjectInfoFromPosition(null, null, objectPosition, false, false
									);
								if (!(aoi is NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeDeletedObjectInfo))
								{
									objectToString = aoi.ToString();
									NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoi = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
										)aoi;
									prevObjectOID = nnoi.GetPreviousObjectOID();
									nextObjectOID = nnoi.GetNextObjectOID();
								}
								else
								{
									objectToString = " deleted";
									prevObjectOID = null;
									nextObjectOID = null;
								}
							}
							catch (System.Exception)
							{
								// info = new IDInfo(currentId, objectPosition,
								// idStatus, blockId, "unknow", "Error", -1, -1);
								// idInfos.add(info);
								objectToString = "?";
								prevObjectOID = null;
								nextObjectOID = null;
							}
						}
						try
						{
							objectType = GetObjectTypeFromPosition(objectPosition);
						}
						catch (System.Exception)
						{
							objectType = "(error?)";
						}
						if (objectTypeToDisplay == null || objectTypeToDisplay.Equals(objectType))
						{
							fsi.SetReadPosition(currentPosition);
							info = new NeoDatis.Odb.Impl.Core.Layers.Layer3.Oid.FullIDInfo(currentId, objectPosition
								, idStatus, blockId, objectType, objectToString, prevObjectOID, nextObjectOID);
							idInfos.Add(info);
						}
					}
					else
					{
						try
						{
							NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci = ReadClassInfoHeader(NeoDatis.Odb.Core.Oid.OIDFactory
								.BuildClassOID(currentId));
							objectType = "Class def. of " + ci.GetFullClassName();
							objectToString = ci.ToString();
							prevObjectOID = ci.GetPreviousClassOID();
							nextObjectOID = ci.GetNextClassOID();
							info = new NeoDatis.Odb.Impl.Core.Layers.Layer3.Oid.FullIDInfo(currentId, objectPosition
								, idStatus, blockId, objectType, objectToString, prevObjectOID, nextObjectOID);
							idInfos.Add(info);
						}
						catch (System.Exception)
						{
							info = new NeoDatis.Odb.Impl.Core.Layers.Layer3.Oid.FullIDInfo(currentId, objectPosition
								, idStatus, blockId, "unknow", "Error", null, null);
							idInfos.Add(info);
						}
					}
					fsi.SetReadPosition(nextRepetitionPosition);
				}
				while (currentId != blockMaxId);
				currentBlockPosition = nextBlockPosition;
			}
			return idInfos;
		}

		public virtual NeoDatis.Odb.OID GetIdOfObjectAt(long position, bool includeDeleted
			)
		{
			fsi.SetReadPosition(position + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Integer
				.GetSize());
			byte blockType = fsi.ReadByte("object block type");
			if (NeoDatis.Odb.Impl.Core.Layers.Layer3.Block.BlockTypes.IsPointer(blockType))
			{
				return GetIdOfObjectAt(fsi.ReadLong("new position"), includeDeleted);
			}
			if (NeoDatis.Odb.Impl.Core.Layers.Layer3.Block.BlockTypes.IsNonNative(blockType))
			{
				return NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(fsi.ReadLong("oid"));
			}
			if (includeDeleted && NeoDatis.Odb.Impl.Core.Layers.Layer3.Block.BlockTypes.IsDeletedObject
				(blockType))
			{
				return NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(fsi.ReadLong("oid"));
			}
			throw new NeoDatis.Odb.CorruptedDatabaseException(NeoDatis.Odb.Core.NeoDatisError
				.WrongTypeForBlockType.AddParameter(NeoDatis.Odb.Impl.Core.Layers.Layer3.Block.BlockTypes
				.BlockTypeNonNativeObject).AddParameter(blockType).AddParameter(position));
		}

		public virtual void Close()
		{
			storageEngine = null;
			blockPositions.Clear();
			blockPositions = null;
		}

		public virtual object BuildOneInstance(NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 objectInfo)
		{
			return instanceBuilder.BuildOneInstance(objectInfo);
		}

		public virtual NeoDatis.Odb.Objects<T> GetObjects<T>(System.Type clazz, bool inMemory
			, int startIndex, int endIndex)
		{
			return GetObjects<T>(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery(clazz), 
				inMemory, startIndex, endIndex);
		}

		public virtual NeoDatis.Odb.Objects<T> GetObjects<T>(string fullClassName, bool inMemory
			, int startIndex, int endIndex)
		{
			return GetObjects<T>(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery(fullClassName
				), inMemory, startIndex, endIndex);
		}

		public virtual NeoDatis.Odb.Objects<T> GetObjects<T>(NeoDatis.Odb.Core.Query.IQuery
			 query, bool inMemory, int startIndex, int endIndex)
		{
			IMatchingObjectAction queryResultAction = new CollectionQueryResultAction<T>(query, inMemory, storageEngine, true, instanceBuilder);
			return QueryManager.GetQueryExecutor(query, storageEngine, instanceBuilder).Execute<T>(inMemory, startIndex, endIndex, true, queryResultAction);
		}

		public virtual NeoDatis.Odb.Values GetValues(NeoDatis.Odb.Core.Query.IValuesQuery
			 valuesQuery, int startIndex, int endIndex)
		{
			NeoDatis.Odb.Core.Query.Execution.IMatchingObjectAction queryResultAction = null;
			if (valuesQuery.HasGroupBy())
			{
				queryResultAction = new NeoDatis.Odb.Impl.Core.Query.Values.GroupByValuesQueryResultAction
					(valuesQuery, storageEngine, instanceBuilder);
			}
			else
			{
				queryResultAction = new NeoDatis.Odb.Impl.Core.Query.Values.ValuesQueryResultAction
					(valuesQuery, storageEngine, instanceBuilder);
			}
			NeoDatis.Odb.Objects<NeoDatis.Odb.ObjectValues> objects = GetObjectInfos<NeoDatis.Odb.ObjectValues>(valuesQuery
				, true, startIndex, endIndex, false, queryResultAction);
			return (NeoDatis.Odb.Values)objects;
		}

		public virtual NeoDatis.Odb.Core.Transaction.ISession GetSession()
		{
			return storageEngine.GetSession(true);
		}

		public virtual NeoDatis.Odb.Objects<T> GetObjectInfos<T>(NeoDatis.Odb.Core.Query.IQuery
			 query, bool inMemory, int startIndex, int endIndex, bool returnObjects, NeoDatis.Odb.Core.Query.Execution.IMatchingObjectAction
			 queryResultAction)
		{
			NeoDatis.Odb.Core.Query.Execution.IQueryExecutor executor = NeoDatis.Odb.Core.Query.QueryManager
				.GetQueryExecutor(query, storageEngine, instanceBuilder);
			return executor.Execute<T>(inMemory, startIndex, endIndex, returnObjects, queryResultAction
				);
		}

		public virtual NeoDatis.Odb.Objects<T> GetObjectInfos<T>(string fullClassName, bool
			 inMemory, int startIndex, int endIndex, bool returnOjects)
		{
			NeoDatis.Odb.Core.Query.IQuery query = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(fullClassName);
			NeoDatis.Odb.Core.Query.Execution.IMatchingObjectAction queryResultAction = new NeoDatis.Odb.Impl.Core.Query.Criteria.CollectionQueryResultAction<T>
				(query, inMemory, storageEngine, returnOjects, instanceBuilder);
			return GetObjectInfos<T>(query, inMemory, startIndex, endIndex, returnOjects, queryResultAction
				);
		}

		public virtual string GetBaseIdentification()
		{
			return storageEngine.GetBaseIdentification().GetIdentification();
		}

		/// <summary>
		/// This is an utility method to get the linked list of All Object Info
		/// Header.
		/// </summary>
		/// <remarks>
		/// This is an utility method to get the linked list of All Object Info
		/// Header. For debug purpose
		/// </remarks>
		/// <param name="classInfo"></param>
		/// <></>
		public virtual NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader
			> GetObjectInfoHeaderList(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo classInfo
			)
		{
			if (classInfo.GetNumberOfObjects() == 0)
			{
				return new NeoDatis.Tool.Wrappers.List.OdbArrayList<NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader
					>();
			}
			NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader
				> list = new NeoDatis.Tool.Wrappers.List.OdbArrayList<NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader
				>((int)classInfo.GetNumberOfObjects());
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader oih = null;
			NeoDatis.Odb.OID oid = classInfo.GetCommitedZoneInfo().first;
			if (oid == null)
			{
				oid = classInfo.GetUncommittedZoneInfo().first;
			}
			while (oid != null)
			{
				oih = ReadObjectInfoHeaderFromOid(oid, true);
				list.Add(oih);
				oid = oih.GetNextObjectOID();
			}
			return list;
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer2.Instance.IInstanceBuilder GetInstanceBuilder
			()
		{
			return instanceBuilder;
		}
	}
}
