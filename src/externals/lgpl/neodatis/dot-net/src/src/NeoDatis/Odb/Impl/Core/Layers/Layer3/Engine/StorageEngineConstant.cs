namespace NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine
{
	/// <summary>Some Storage engine constants about offset position for object writing/reading.
	/// 	</summary>
	/// <remarks>Some Storage engine constants about offset position for object writing/reading.
	/// 	</remarks>
	public class StorageEngineConstant
	{
		/// <summary>Used to make an attribute reference a null object - setting its id to zero
		/// 	</summary>
		public static readonly NeoDatis.Odb.OID NullObjectId = null;

		public const long NullObjectIdId = 0;

		public const long DeletedObjectPosition = 0;

		public const long NullObjectPosition = 0;

		public const long ObjectIsNotInCache = -1;

		public const long PositionNotInitialized = -1;

		public const long ObjectDoesNotExist = -2;

		/// <summary>this occurs when a class has been refactored adding a field.</summary>
		/// <remarks>this occurs when a class has been refactored adding a field. Old objects do not the new field
		/// 	</remarks>
		public const long FieldDoesNotExist = -1;

		public const byte Version2 = 2;

		public const byte Version3 = 3;

		public const byte Version4 = 4;

		public const byte Version5 = 5;

		public const byte Version6 = 6;

		public const byte Version7 = 7;

		public const int Version8 = 8;

		/// <summary>1.9 file format</summary>
		public const int Version9 = 9;

		public const int CurrentFileFormatVersion = Version9;

		/// <summary>Use Encryption : 1 byte)</summary>
		public const int DatabaseHeaderUseEncryptionPosition = 0;

		/// <summary>File format version : 1 int (4 bytes)</summary>
		public static readonly int DatabaseHeaderVersionPosition = NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
			.Byte.GetSize();

		/// <summary>Future flag , may be to keep programming language that created the database: 1 byte
		/// 	</summary>
		public static readonly int DatabaseHeaderLanguageIdPosition = NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
			.Integer.GetSize();

		/// <summary>The Database ID : 4 Long (4*8 bytes)</summary>
		public static readonly int DatabaseHeaderDatabaseIdPosition = DatabaseHeaderLanguageIdPosition
			 + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Byte.GetSize();

		/// <summary>To indicate if database uses replication : 1 byte)</summary>
		public static readonly int DatabaseHeaderUseReplicationPosition = DatabaseHeaderDatabaseIdPosition
			 + 4 * NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Long.GetSize();

		/// <summary>The last Transaction ID 2 long (2*4*8 bytes)</summary>
		public static readonly int DatabaseHeaderLastTransactionId = DatabaseHeaderUseReplicationPosition
			 + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Byte.GetSize();

		/// <summary>The number of classes in the meta model 1 long (4*8 bytes)</summary>
		public static readonly int DatabaseHeaderNumberOfClassesPosition = DatabaseHeaderLastTransactionId
			 + 2 * NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Long.GetSize();

		/// <summary>The first class OID : 1 Long (8 bytes)</summary>
		public static readonly int DatabaseHeaderFirstClassOid = DatabaseHeaderNumberOfClassesPosition
			 + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Long.GetSize();

		/// <summary>The last ODB close status.</summary>
		/// <remarks>The last ODB close status. Used to detect if the transaction is ok : 1 byte
		/// 	</remarks>
		public static readonly int DatabaseHeaderLastCloseStatusPosition = DatabaseHeaderFirstClassOid
			 + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Long.GetSize();

		/// <summary>The Database character encoding : 50 bytes</summary>
		public static readonly int DatabaseHeaderDatabaseCharacterEncodingPosition = DatabaseHeaderLastCloseStatusPosition
			 + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Byte.GetSize();

		/// <summary>To indicate if database is password protected : 1 byte</summary>
		public static readonly int DatabaseHeaderDatabaseIsUserProtected = DatabaseHeaderDatabaseCharacterEncodingPosition
			 + 58 * NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Byte.GetSize();

		/// <summary>The database user name : 50 bytes</summary>
		public static readonly int DatabaseHeaderDatabaseUserName = DatabaseHeaderDatabaseIsUserProtected
			 + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Byte.GetSize();

		/// <summary>The database password : 50 bytes</summary>
		public static readonly int DatabaseHeaderDatabasePassword = DatabaseHeaderDatabaseUserName
			 + 58 * NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Byte.GetSize();

		/// <summary>The position of the current id block: 1 long</summary>
		public static readonly int DatabaseHeaderCurrentIdBlockPosition = DatabaseHeaderDatabasePassword
			 + 58 * NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Byte.GetSize();

		/// <summary>First ID Block position</summary>
		public static readonly int DatabaseHeaderFirstIdBlockPosition = DatabaseHeaderCurrentIdBlockPosition
			 + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Long.GetSize();

		public static readonly int DatabaseHeaderProtectedZoneSize = DatabaseHeaderCurrentIdBlockPosition;

		public static readonly int[] DatabaseHeaderPositions = new int[] { DatabaseHeaderUseEncryptionPosition
			, DatabaseHeaderVersionPosition, DatabaseHeaderLanguageIdPosition, DatabaseHeaderDatabaseIdPosition
			, DatabaseHeaderUseReplicationPosition, DatabaseHeaderLastTransactionId, DatabaseHeaderNumberOfClassesPosition
			, DatabaseHeaderFirstClassOid, DatabaseHeaderLastCloseStatusPosition, DatabaseHeaderDatabaseCharacterEncodingPosition
			, DatabaseHeaderDatabaseIsUserProtected, DatabaseHeaderDatabasePassword, DatabaseHeaderCurrentIdBlockPosition
			 };

		public const long ClassOffsetBlockSize = 0;

		public static readonly long ClassOffsetBlockType = ClassOffsetBlockSize + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
			.Integer.GetSize();

		public static readonly long ClassOffsetCategory = ClassOffsetBlockType + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
			.Byte.GetSize();

		public static readonly long ClassOffsetId = ClassOffsetCategory + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
			.Byte.GetSize();

		public static readonly long ClassOffsetPreviousClassPosition = ClassOffsetId + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
			.Long.GetSize();

		public static readonly long ClassOffsetNextClassPosition = ClassOffsetPreviousClassPosition
			 + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Long.GetSize();

		public static readonly long ClassOffsetClassNbObjects = ClassOffsetNextClassPosition
			 + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Long.GetSize();

		public static readonly long ClassOffsetClassFirstObjectPosition = ClassOffsetClassNbObjects
			 + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Long.GetSize();

		public static readonly long ClassOffsetClassLastObjectPosition = ClassOffsetClassFirstObjectPosition
			 + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Long.GetSize();

		public static readonly long ClassOffsetFullClassNameSize = ClassOffsetNextClassPosition
			 + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Long.GetSize();

		public const long ObjectOffsetBlockSize = 0;

		public static readonly long ObjectOffsetBlockType = ObjectOffsetBlockSize + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
			.Integer.GetSize();

		public static readonly long ObjectOffsetObjectId = ObjectOffsetBlockType + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
			.Byte.GetSize();

		public static readonly long ObjectOffsetClassInfoId = ObjectOffsetObjectId + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
			.Long.GetSize();

		public static readonly long ObjectOffsetPreviousObjectOid = ObjectOffsetClassInfoId
			 + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Long.GetSize();

		public static readonly long ObjectOffsetNextObjectOid = ObjectOffsetPreviousObjectOid
			 + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Long.GetSize();

		public static readonly long ObjectOffsetCreationDate = ObjectOffsetNextObjectOid 
			+ NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Long.GetSize();

		public static readonly long ObjectOffsetUpdateDate = ObjectOffsetCreationDate + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
			.Long.GetSize();

		public static readonly long ObjectOffsetVersion = ObjectOffsetUpdateDate + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
			.Long.GetSize();

		public static readonly long ObjectOffsetReferencePointer = ObjectOffsetVersion + 
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Integer.GetSize();

		public static readonly long ObjectOffsetIsExternallySynchronized = ObjectOffsetReferencePointer
			 + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Long.GetSize();

		public static readonly long ObjectOffsetNbAttributes = ObjectOffsetIsExternallySynchronized
			 + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeBoolean.GetSize();

		/// <summary>
		/// <pre>
		/// ID Block Header :
		/// Block size             : 1 int
		/// Block type             : 1 byte
		/// Block status           : 1 byte
		/// Prev block position    : 1 long
		/// Next block position    : 1 long
		/// Block number           : 1 int
		/// Max id                 : 1 long
		/// Total size = 34
		/// </pre>
		/// </summary>
		public static readonly long BlockIdOffsetForBlockStatus = NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
			.Integer.GetSize() + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Byte.GetSize();

		public static readonly long BlockIdOffsetForPrevBlock = BlockIdOffsetForBlockStatus
			 + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Byte.GetSize();

		public static readonly long BlockIdOffsetForNextBlock = BlockIdOffsetForPrevBlock
			 + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Long.GetSize();

		public static readonly long BlockIdOffsetForBlockNumber = BlockIdOffsetForNextBlock
			 + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Long.GetSize();

		public static readonly long BlockIdOffsetForMaxId = BlockIdOffsetForBlockNumber +
			 NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Integer.GetSize();

		public static readonly long BlockIdOffsetForStartOfRepetition = BlockIdOffsetForMaxId
			 + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Long.GetSize();

		/// <summary>pull id type (byte),id(long),</summary>
		public const long BlockIdRepetitionIdType = 0;

		public static readonly long BlockIdRepetitionId = BlockIdRepetitionIdType + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
			.Byte.GetSize();

		public static readonly long BlockIdRepetitionIdStatus = BlockIdRepetitionId + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
			.Long.GetSize();

		public static readonly long BlockIdRepetitionObjectPosition = BlockIdRepetitionIdStatus
			 + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Byte.GetSize();

		public const long NativeObjectOffsetBlockSize = 0;

		public static readonly long NativeObjectOffsetBlockType = NativeObjectOffsetBlockSize
			 + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Integer.GetSize();

		public static readonly long NativeObjectOffsetOdbTypeId = NativeObjectOffsetBlockType
			 + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Byte.GetSize();

		public static readonly long NativeObjectOffsetObjectIsNull = NativeObjectOffsetOdbTypeId
			 + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Integer.GetSize();

		public static readonly long NativeObjectOffsetDataArea = NativeObjectOffsetObjectIsNull
			 + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Boolean.GetSize();

		public const byte NoEncryption = 0;

		public const byte WithEncryption = 1;

		public const byte NoReplication = 0;

		public const byte WithReplication = 1;

		public static readonly string NoEncoding = "no-encoding";
		// TODO Something is wrong here : two constant with the same value!!*/
		// ********************************************************
		// DATABASE HEADER
		// ********************************************************
		// **********************************************************
		// END OF DATABASE HEADER
		// *********************************************************
		// CLASS OFFSETS
		//OBJECT OFFSETS - update this section when modifying the odb file format 
		// Encryption flag
		// Replication flag
	}
}
