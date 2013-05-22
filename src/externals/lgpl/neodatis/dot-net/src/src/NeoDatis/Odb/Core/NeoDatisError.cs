namespace NeoDatis.Odb.Core
{
	/// <summary>All NeoDatis ODB Errors.</summary>
	/// <remarks>
	/// All NeoDatis ODB Errors. Errors can be user errors or Internal errors. All @1
	/// in error description will be replaced by parameters
	/// </remarks>
	/// <1author>olivier s</1author>
	public class NeoDatisError : NeoDatis.Odb.Core.IError
	{
		private int code;

		private string description;

		private NeoDatis.Tool.Wrappers.List.IOdbList<object> parameters;

		public static readonly NeoDatis.Odb.Core.NeoDatisError NullNextObjectOid = new NeoDatis.Odb.Core.NeoDatisError
			(100, "ODB has detected an inconsistency while reading instance(of @1) #@2 over @3 with oid @4 which has a null 'next object oid'"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError InstancePositionOutOfFile = 
			new NeoDatis.Odb.Core.NeoDatisError(101, "ODB is trying to read an instance at position @1 which is out of the file - File size is @2"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError InstancePositionIsNegative
			 = new NeoDatis.Odb.Core.NeoDatisError(102, "ODB is trying to read an instance at a negative position @1 , oid=@2 : @3"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError WrongTypeForBlockType = new 
			NeoDatis.Odb.Core.NeoDatisError(201, "Block type of wrong type : expected @1, Found @2 at position @3"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError WrongBlockSize = new NeoDatis.Odb.Core.NeoDatisError
			(202, "Wrong Block size : expected @1, Found @2 at position @3");

		public static readonly NeoDatis.Odb.Core.NeoDatisError WrongOidAtPosition = new NeoDatis.Odb.Core.NeoDatisError
			(203, "Reading object with oid @1 at position @2, but found oid @3");

		public static readonly NeoDatis.Odb.Core.NeoDatisError BlockNumberDoesExist = new 
			NeoDatis.Odb.Core.NeoDatisError(205, "Block(of ids) with number @1 does not exist"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError FoundPointer = new NeoDatis.Odb.Core.NeoDatisError
			(204, "Found a pointer for oid @1 at position @2");

		public static readonly NeoDatis.Odb.Core.NeoDatisError ObjectIsMarkedAsDeletedForOid
			 = new NeoDatis.Odb.Core.NeoDatisError(206, "Object with oid @1 is marked as deleted"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError ObjectIsMarkedAsDeletedForPosition
			 = new NeoDatis.Odb.Core.NeoDatisError(207, "Object with position @1 is marked as deleted"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError NativeTypeNotSupported = new 
			NeoDatis.Odb.Core.NeoDatisError(208, "Native type not supported @1 @2");

		public static readonly NeoDatis.Odb.Core.NeoDatisError NativeTypeDivergence = new 
			NeoDatis.Odb.Core.NeoDatisError(209, "Native type informed(@1) is different from the one informed (@2)"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError NegativeClassNumberInHeader
			 = new NeoDatis.Odb.Core.NeoDatisError(210, "number of classes is negative while reading database header : @1 at position @2"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError UnknownBlockType = new NeoDatis.Odb.Core.NeoDatisError
			(211, "Unknown block type @1 at @2");

		public static readonly NeoDatis.Odb.Core.NeoDatisError UnsupportedIoType = new NeoDatis.Odb.Core.NeoDatisError
			(212, "Unsupported IO Type : @1");

		public static readonly NeoDatis.Odb.Core.NeoDatisError ObjectDoesNotExistInCache = 
			new NeoDatis.Odb.Core.NeoDatisError(213, "Object does not exist in cache");

		public static readonly NeoDatis.Odb.Core.NeoDatisError ObjectWithOidDoesNotExistInCache
			 = new NeoDatis.Odb.Core.NeoDatisError(213, "Object with oid @1 does not exist in cache"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError ObjectInfoNotInTempCache = 
			new NeoDatis.Odb.Core.NeoDatisError(214, "ObjectInfo does not exist in temporary cache oid=@1 and position=@2"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError CanNotDeleteFile = new NeoDatis.Odb.Core.NeoDatisError
			(215, "Can not delete file @1");

		public static readonly NeoDatis.Odb.Core.NeoDatisError GoToPosition = new NeoDatis.Odb.Core.NeoDatisError
			(216, "Error while going to position @1, length = @2");

		public static readonly NeoDatis.Odb.Core.NeoDatisError ErrorInCoreProviderInitialization
			 = new NeoDatis.Odb.Core.NeoDatisError(217, "Error while initializing CoreProvider @1"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError UndefinedClassInfo = new NeoDatis.Odb.Core.NeoDatisError
			(218, "Undefined class info for @1");

		public static readonly NeoDatis.Odb.Core.NeoDatisError AbstractObjectInfoTypeNotSupported
			 = new NeoDatis.Odb.Core.NeoDatisError(219, "Abstract Object Info type not supported : @1"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError NegativeBlockSize = new NeoDatis.Odb.Core.NeoDatisError
			(220, "Negative block size at @1 : size = @2, object=@3");

		public static readonly NeoDatis.Odb.Core.NeoDatisError InplaceUpdateNotPossibleForArray
			 = new NeoDatis.Odb.Core.NeoDatisError(221, "Array in place update with array smaller than element array index to update : array size=@1, element index=@2"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError OperationNotImplemented = 
			new NeoDatis.Odb.Core.NeoDatisError(222, "Operation not supported : @1");

		public static readonly NeoDatis.Odb.Core.NeoDatisError InstanceBuilderWrongObjectType
			 = new NeoDatis.Odb.Core.NeoDatisError(223, "Wrong type of object: expecting @1 and received @2"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError InstanceBuilderWrongObjectContainerType
			 = new NeoDatis.Odb.Core.NeoDatisError(224, "Building instance of @1 : can not put a @2 into a @3"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError InstanceBuilderNativeTypeInCollectionNotSupported
			 = new NeoDatis.Odb.Core.NeoDatisError(225, "Native @1 in Collection(List,array,Map) not supported"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError ObjectIntrospectorNoFieldWithName
			 = new NeoDatis.Odb.Core.NeoDatisError(226, "Class/Interface @1 does not have attribute '@2'"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError ObjectIntrospectorClassNotFound
			 = new NeoDatis.Odb.Core.NeoDatisError(227, "Class not found : @1");

		public static readonly NeoDatis.Odb.Core.NeoDatisError ClassPoolCreateClass = new 
			NeoDatis.Odb.Core.NeoDatisError(228, "Error while creating (reflection) class @1"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError BufferTooSmall = new NeoDatis.Odb.Core.NeoDatisError
			(229, "Buffer too small: buffer size = @1 and data size = @2 - should not happen"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError FileInterfaceWriteBytesNotImplementedForTransaction
			 = new NeoDatis.Odb.Core.NeoDatisError(230, "writeBytes not implemented for transactions"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError FileInterfaceReadError = new 
			NeoDatis.Odb.Core.NeoDatisError(231, "Error reading @1 bytes at @2 : read @3 bytes instead"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError PointerToSelf = new NeoDatis.Odb.Core.NeoDatisError
			(232, "Error while creating a pointer : a pointer to itself : @1 -> @2 for oid @3"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError IndexNotFound = new NeoDatis.Odb.Core.NeoDatisError
			(233, "No index defined on class @1 at index position @2");

		public static readonly NeoDatis.Odb.Core.NeoDatisError NotYetImplemented = new NeoDatis.Odb.Core.NeoDatisError
			(234, "Not yet implemented : @1");

		public static readonly NeoDatis.Odb.Core.NeoDatisError MetaModelClassNameDoesNotExist
			 = new NeoDatis.Odb.Core.NeoDatisError(235, "Class @1 does not exist in meta-model"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError MetaModelClassWithOidDoesNotExist
			 = new NeoDatis.Odb.Core.NeoDatisError(236, "Class with oid @1 does not exist in meta-model"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError MetaModelClassWithPositionDoesNotExist
			 = new NeoDatis.Odb.Core.NeoDatisError(237, "Class with position @1 does not exist in meta-model"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError ClassInfoDoNotHaveTheAttribute
			 = new NeoDatis.Odb.Core.NeoDatisError(238, "Class @1 does not have attribute with name @2 in the database meta-model"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError OdbTypeIdDoesNotExist = new 
			NeoDatis.Odb.Core.NeoDatisError(239, "ODBtype with id @1 does not exist");

		public static readonly NeoDatis.Odb.Core.NeoDatisError OdbTypeNativeTypeWithIdDoesNotExist
			 = new NeoDatis.Odb.Core.NeoDatisError(240, "Native type with id @1 does not exist"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError QueryEngineNotSet = new NeoDatis.Odb.Core.NeoDatisError
			(241, "Storage engine not set on query");

		public static readonly NeoDatis.Odb.Core.NeoDatisError QueryTypeNotImplemented = 
			new NeoDatis.Odb.Core.NeoDatisError(242, "Query type @1 not implemented");

		public static readonly NeoDatis.Odb.Core.NeoDatisError CryptoAlgorithNotFound = new 
			NeoDatis.Odb.Core.NeoDatisError(243, "Could not get the MD5 algorithm to encrypt the password"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError XmlHeader = new NeoDatis.Odb.Core.NeoDatisError
			(244, "Error while creating XML Header");

		public static readonly NeoDatis.Odb.Core.NeoDatisError XmlReservingIds = new NeoDatis.Odb.Core.NeoDatisError
			(245, "Error while reserving @1 ids");

		public static readonly NeoDatis.Odb.Core.NeoDatisError XmlSettingMetaModel = new 
			NeoDatis.Odb.Core.NeoDatisError(246, "Error while setting meta model");

		public static readonly NeoDatis.Odb.Core.NeoDatisError SerializationFromString = 
			new NeoDatis.Odb.Core.NeoDatisError(247, "Error while deserializing: expecting classId @1 and received @2"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError SerializationCollection = 
			new NeoDatis.Odb.Core.NeoDatisError(248, "Error while deserializing collection: sizes are not consistent : expected @1, found @2"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError MetamodelReadingLastObject
			 = new NeoDatis.Odb.Core.NeoDatisError(249, "Error while reading last object of type @1 at with OID @2"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError CacheNegativeOid = new NeoDatis.Odb.Core.NeoDatisError
			(250, "Negative oid set in cache @1");

		public static readonly NeoDatis.Odb.Core.NeoDatisError ClientServerSynchronizeIds
			 = new NeoDatis.Odb.Core.NeoDatisError(251, "Error while synchronizing oids,length are <>, local=@1, client=@2"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError ClientServerCanNotOpenOdbServerOnPort
			 = new NeoDatis.Odb.Core.NeoDatisError(252, "Can not start ODB server on port @1"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError ClientServerCanNotAssociateOids
			 = new NeoDatis.Odb.Core.NeoDatisError(253, "Can not associate server and client oids : server oid=@1 and client oid=@2"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError SessionDoesNotExistForConnection
			 = new NeoDatis.Odb.Core.NeoDatisError(254, "Connection @1 for base @2 does not have any associated session"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError SessionDoesNotExistForConnectionId
			 = new NeoDatis.Odb.Core.NeoDatisError(255, "Connection ID @1 does not have any associated session"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError ClientServerError = new NeoDatis.Odb.Core.NeoDatisError
			(256, "ServerSide Error : @1");

		public static readonly NeoDatis.Odb.Core.NeoDatisError ObjectReaderDirectCall = new 
			NeoDatis.Odb.Core.NeoDatisError(257, "Generic readObjectInfo called for non native object info"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError CacheObjectInfoHeaderWithoutClassId
			 = new NeoDatis.Odb.Core.NeoDatisError(258, "Object Info Header without class id ; oih.oid=@1"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError NonNativeAttributeStoredByPositionInsteadOfOid
			 = new NeoDatis.Odb.Core.NeoDatisError(259, "Non native attribute (@1) of class @2 stored by position @3 instead of oid"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError CacheNullOid = new NeoDatis.Odb.Core.NeoDatisError
			(260, "Null OID");

		public static readonly NeoDatis.Odb.Core.NeoDatisError NegativePosition = new NeoDatis.Odb.Core.NeoDatisError
			(261, "Negative position : @1");

		public static readonly NeoDatis.Odb.Core.NeoDatisError UnexpectedSituation = new 
			NeoDatis.Odb.Core.NeoDatisError(262, "Unexpected situation: @1");

		public static readonly NeoDatis.Odb.Core.NeoDatisError ImportError = new NeoDatis.Odb.Core.NeoDatisError
			(263, "Import error: @1");

		public static readonly NeoDatis.Odb.Core.NeoDatisError ClientServerCanNotCreateClassInfo
			 = new NeoDatis.Odb.Core.NeoDatisError(264, "ServerSide Error : Can not create class info @1"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError ClientServerMetaModelInconsistency
			 = new NeoDatis.Odb.Core.NeoDatisError(265, "ServerSide Error : Meta model on server and client are inconsistent : class @1 exist on server and does not exist on the client!"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError ClientServerMetaModelInconsistencyDifferentOid
			 = new NeoDatis.Odb.Core.NeoDatisError(266, "ServerSide Error : Meta model on server and client are inconsistent : class @1 have different OIDs on server (@2) and client(@3)!"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError MethodShouldNotBeCalled = 
			new NeoDatis.Odb.Core.NeoDatisError(267, "Method @1 should not be called on @2");

		public static readonly NeoDatis.Odb.Core.NeoDatisError CacheNegativePosition = new 
			NeoDatis.Odb.Core.NeoDatisError(268, "Caching an ObjectInfoHeader with negative position @1"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError ErrorWhileGettingObjectFromListAtIndex
			 = new NeoDatis.Odb.Core.NeoDatisError(269, "Error while getting object from list at index @1"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError ClassInfoDoesNotExistInMetaModel
			 = new NeoDatis.Odb.Core.NeoDatisError(270, "Class Info @1 does not exist in MetaModel"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError BtreeSizeDiffersFromClassElementNumber
			 = new NeoDatis.Odb.Core.NeoDatisError(271, "The Index has @1 element(s) whereas the Class has @2 objects. The two values should be equal"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError ClientServerConnectionIsNull
			 = new NeoDatis.Odb.Core.NeoDatisError(272, "The connection ID @1 does not exist in connection manager (@2)"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError ClientServerPortIsBusy = new 
			NeoDatis.Odb.Core.NeoDatisError(273, "Can not start ODB server on port @1: The port is busy. Check if another server is not already running of this port"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError InstanceBuilderNativeType = 
			new NeoDatis.Odb.Core.NeoDatisError(274, "Native object of type @1 can not be instanciated"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError ClassIntrospectionError = 
			new NeoDatis.Odb.Core.NeoDatisError(275, "Class Introspectpr error for class @1"
			);

		public static readonly NeoDatis.Odb.Core.IError EndOfFileReached = new NeoDatis.Odb.Core.NeoDatisError
			(276, "End Of File reached - position = @1 : Length = @2");

		public static readonly NeoDatis.Odb.Core.IError MapInstanciationError = new NeoDatis.Odb.Core.NeoDatisError
			(277, "Error while creating instance of MAP of class @1");

		public static readonly NeoDatis.Odb.Core.IError CollectionInstanciationError = new 
			NeoDatis.Odb.Core.NeoDatisError(278, "Error while creating instance of Collection of class @1"
			);

		public static readonly NeoDatis.Odb.Core.IError InstanciationError = new NeoDatis.Odb.Core.NeoDatisError
			(279, "Error while creating instance of type @1");

		public static readonly NeoDatis.Odb.Core.IError ServerSideError = new NeoDatis.Odb.Core.NeoDatisError
			(280, "Server side error @1 : @2");

		public static readonly NeoDatis.Odb.Core.IError NetSerialisationError = new NeoDatis.Odb.Core.NeoDatisError
			(281, "Net Serialization Error : @1 \n@2");

		public static readonly NeoDatis.Odb.Core.IError ClientNetError = new NeoDatis.Odb.Core.NeoDatisError
			(282, "Client Net Error");

		public static readonly NeoDatis.Odb.Core.IError ServerNetError = new NeoDatis.Odb.Core.NeoDatisError
			(283, "Server Net Error");

		public static readonly NeoDatis.Odb.Core.IError ErrorWhileGettingConstrctorsOfClass
			 = new NeoDatis.Odb.Core.NeoDatisError(284, "Error while getting constructor of @1"
			);

		public static readonly NeoDatis.Odb.Core.IError UnknownHost = new NeoDatis.Odb.Core.NeoDatisError
			(285, "Unknown host");

		public static readonly NeoDatis.Odb.Core.NeoDatisError CacheNullObject = new NeoDatis.Odb.Core.NeoDatisError
			(286, "Null Object : @1");

		public static readonly NeoDatis.Odb.Core.NeoDatisError LookupKeyNotFound = new NeoDatis.Odb.Core.NeoDatisError
			(287, "Lookup key not found : @1");

		public static readonly NeoDatis.Odb.Core.NeoDatisError ServerError = new NeoDatis.Odb.Core.NeoDatisError
			(288, "Server error : @1");

		public static readonly NeoDatis.Odb.Core.NeoDatisError ReflectionErrorWhileGettingField
			 = new NeoDatis.Odb.Core.NeoDatisError(289, "Error while getting field @1 on class @2"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError NotYetSupported = new NeoDatis.Odb.Core.NeoDatisError
			(290, "Not Yet Supported : @1");

		public static readonly NeoDatis.Odb.Core.NeoDatisError FileNotFound = new NeoDatis.Odb.Core.NeoDatisError
			(291, "File not found: @1");

		public static readonly NeoDatis.Odb.Core.NeoDatisError IndexIsCorrupted = new NeoDatis.Odb.Core.NeoDatisError
			(292, "Index '@1' of class '@2' is corrupted: class has @3 objects, index has @4 entries"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError ErrorWhileCreatingMessageStreamer
			 = new NeoDatis.Odb.Core.NeoDatisError(293, "Error while creating message streamer '@1'"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError ClientServerUnknownCommand
			 = new NeoDatis.Odb.Core.NeoDatisError(294, "Unknown server command : @1");

		public static readonly NeoDatis.Odb.Core.NeoDatisError ContainsQueryWithNoQuery = 
			new NeoDatis.Odb.Core.NeoDatisError(295, "Contains criteria with no query!");

		public static readonly NeoDatis.Odb.Core.NeoDatisError ContainsQueryWithNoStorageEngine
			 = new NeoDatis.Odb.Core.NeoDatisError(296, "Contains criteria with no engine!");

		public static readonly NeoDatis.Odb.Core.NeoDatisError CrossSessionCacheNullOidForObject
			 = new NeoDatis.Odb.Core.NeoDatisError(297, "Cross session cache does not know the object @1"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError ErrorWhileGettingIpAddress
			 = new NeoDatis.Odb.Core.NeoDatisError(298, "Error while getting IP address of @1"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError CriteriaQueryUnknownAttribute
			 = new NeoDatis.Odb.Core.NeoDatisError(1000, "Attribute @1 used in criteria queria does not exist on class @2"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError RuntimeIncompatibleVersion
			 = new NeoDatis.Odb.Core.NeoDatisError(1001, "Incompatible ODB Version : ODB file version is @1 and Runtime version is @2"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError IncompatibleMetamodel = new 
			NeoDatis.Odb.Core.NeoDatisError(1002, "Incompatible meta-model : @1");

		public static readonly NeoDatis.Odb.Core.NeoDatisError IncompatibleJavaVm = new NeoDatis.Odb.Core.NeoDatisError
			(1003, "Incompatible java virtual Machine, 1.5 or greater is required, you are using : @1"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError OdbIsClosed = new NeoDatis.Odb.Core.NeoDatisError
			(1004, "ODB session has already been closed (@1)");

		public static readonly NeoDatis.Odb.Core.NeoDatisError OdbHasBeenRollbacked = new 
			NeoDatis.Odb.Core.NeoDatisError(1005, "ODB session has been rollbacked (@1)");

		public static readonly NeoDatis.Odb.Core.NeoDatisError OdbCanNotStoreNullObject = 
			new NeoDatis.Odb.Core.NeoDatisError(1006, "ODB can not store null object");

		public static readonly NeoDatis.Odb.Core.NeoDatisError OdbCanNotStoreArrayDirectly
			 = new NeoDatis.Odb.Core.NeoDatisError(1007, "ODB can not store array directly : @1"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError OdbCanNotStoreNativeObjectDirectly
			 = new NeoDatis.Odb.Core.NeoDatisError(1008, "NeoDats ODB can not store native object direclty : @1 which is or seems to be a @2. Workaround: Wrap class @3 into another class"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError ObjectDoesNotExistInCacheForDelete
			 = new NeoDatis.Odb.Core.NeoDatisError(1009, "The object being deleted does not exist in cache. Make sure the object has been loaded before deleting : type=@1 object=[@2]"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError TransactionIsPending = new 
			NeoDatis.Odb.Core.NeoDatisError(1010, "There are pending work associated to current transaction, a commit or rollback should be executed : session id = @1"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError UnknownObjectToGetOid = new 
			NeoDatis.Odb.Core.NeoDatisError(1011, "Unknown object @1");

		public static readonly NeoDatis.Odb.Core.NeoDatisError OdbCanNotReturnOidOfNullObject
			 = new NeoDatis.Odb.Core.NeoDatisError(1012, "Can not return the oid of a null object"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError OdbFileIsLockedByCurrentVirtualMachine
			 = new NeoDatis.Odb.Core.NeoDatisError(1013, "@1 file is locked by the current Virtual machine - check if the database has not been opened in the current VM! : thread = @2 - using multi thread ? @3"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError OdbFileIsLockedByExternalProgram
			 = new NeoDatis.Odb.Core.NeoDatisError(1014, "@1 file is locked - check if the database file is not opened in another program! : thread = @2 - using multi thread ? @3"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError UserNameTooLong = new NeoDatis.Odb.Core.NeoDatisError
			(1015, "User name @1 is too long, should be lesser than 20 characters");

		public static readonly NeoDatis.Odb.Core.NeoDatisError PasswordTooLong = new NeoDatis.Odb.Core.NeoDatisError
			(1016, "Password is too long, it must be less than 20 character long");

		public static readonly NeoDatis.Odb.Core.NeoDatisError TransactionAlreadyCommitedOrRollbacked
			 = new NeoDatis.Odb.Core.NeoDatisError(1017, "Transaction have already been 'committed' or 'rollbacked'"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError DifferentSizeInWriteAction
			 = new NeoDatis.Odb.Core.NeoDatisError(1018, "Size difference in WriteAction.persist :(calculated,stored)=(@1,@2)"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError ClassWithoutConstructor = 
			new NeoDatis.Odb.Core.NeoDatisError(1019, "Class without any constructor : @1");

		public static readonly NeoDatis.Odb.Core.NeoDatisError NoNullableConstructor = new 
			NeoDatis.Odb.Core.NeoDatisError(1020, "Constructor @1 of class @2 was called with null values because it does not have default constructor and it seems the constructor is not prepared for this!"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError QueryBadCriteria = new NeoDatis.Odb.Core.NeoDatisError
			(1021, "CollectionSizeCriteria only work with Collection or Array, and you passed a @1 instead"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError QueryCollectionSizeCriteriaNotSupported
			 = new NeoDatis.Odb.Core.NeoDatisError(1022, "CollectionSizeCriterion sizeType @1 not yet implemented"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError QueryComparableCriteriaAppliedOnNonComparable
			 = new NeoDatis.Odb.Core.NeoDatisError(1023, "ComparisonCriteria with greater than only work with Comparable, and you passed a @1 instead"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError QueryUnknownOperator = new 
			NeoDatis.Odb.Core.NeoDatisError(1024, "Unknow operator @1");

		public static readonly NeoDatis.Odb.Core.NeoDatisError QueryContainsCriterionTypeNotSupported
			 = new NeoDatis.Odb.Core.NeoDatisError(1025, "Where.contain can not be used with a @1, only collections and arrays are supported"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError QueryAttributeTypeNotSupportedInLikeExpression
			 = new NeoDatis.Odb.Core.NeoDatisError(1026, "LikeCriteria with like expression(%) only work with String, and you passed a @1 instead"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError IndexKeysMustImplementComparable
			 = new NeoDatis.Odb.Core.NeoDatisError(1027, "Unable to build index key for attribute that does not implement 'Comparable/IComparable' : Index=@1, attribute = @2 , type = @3"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError QueryNqMatchMethodNotImplemented
			 = new NeoDatis.Odb.Core.NeoDatisError(1029, "ISimpleNativeQuery implementing classes must implement method: boolean match(?Object obj), class @1 does not"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError QueryNqExceptionRaisedByNativeQueryExecution
			 = new NeoDatis.Odb.Core.NeoDatisError(1030, "Exception raised by the native query @1 match method"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError OdbCanNotReturnOidOfUnknownObject
			 = new NeoDatis.Odb.Core.NeoDatisError(1031, "Can not return the oid of a not previously loaded object : @1"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError ErrorWhileAddingObjectToHashmap
			 = new NeoDatis.Odb.Core.NeoDatisError(1032, "Internal error in user object of class @1 in equals or hashCode method : @2"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError AttributeReferencesADeletedObject
			 = new NeoDatis.Odb.Core.NeoDatisError(1033, "Object of type @1 with oid @2 has the attribute '@3' that references a deleted object"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError BeforeDeleteTriggerHasThrownException
			 = new NeoDatis.Odb.Core.NeoDatisError(1034, "Before Delete Trigger @1 has thrown exception. ODB has ignored it \n<user exception>\n@2</user exception>"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError AfterDeleteTriggerHasThrownException
			 = new NeoDatis.Odb.Core.NeoDatisError(1035, "After Delete Trigger @1 has thrown exception. ODB has ignored it\n<user exception>\n@2</user exception>"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError BeforeUpdateTriggerHasThrownException
			 = new NeoDatis.Odb.Core.NeoDatisError(1036, "Before Update Trigger @1 has thrown exception. ODB has ignored it\n<user exception>\n@2</user exception>"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError AfterUpdateTriggerHasThrownException
			 = new NeoDatis.Odb.Core.NeoDatisError(1037, "After Update Trigger @1 has thrown exception. ODB has ignored it\n<user exception>\n@2</user exception>"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError BeforeInsertTriggerHasThrownException
			 = new NeoDatis.Odb.Core.NeoDatisError(1038, "Before Insert Trigger @1 has thrown exception. ODB has ignored it\n<user exception>\n@2</user exception>"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError AfterInsertTriggerHasThrownException
			 = new NeoDatis.Odb.Core.NeoDatisError(1039, "After Insert Trigger @1 has thrown exception. ODB has ignored it\n<user exception>\n@2</user exception>"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError NoMoreObjectsInCollection = 
			new NeoDatis.Odb.Core.NeoDatisError(1040, "No more objects in collection");

		public static readonly NeoDatis.Odb.Core.NeoDatisError IndexAlreadyExist = new NeoDatis.Odb.Core.NeoDatisError
			(1041, "Index @1 already exist on class @2");

		public static readonly NeoDatis.Odb.Core.NeoDatisError IndexDoesNotExist = new NeoDatis.Odb.Core.NeoDatisError
			(1042, "Index @1 does not exist on class @2");

		public static readonly NeoDatis.Odb.Core.NeoDatisError QueryAttributeTypeNotSupportedInIequalExpression
			 = new NeoDatis.Odb.Core.NeoDatisError(1043, "EqualCriteria with case insensitive expression only work with String, and you passed a @1 instead"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError ValuesQueryAliasDoesNotExist
			 = new NeoDatis.Odb.Core.NeoDatisError(1044, "Alias @1 does not exist in query result. Existing alias are @2"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError ValuesQueryNotConsistent = 
			new NeoDatis.Odb.Core.NeoDatisError(1045, "Single row actions (like sum,count,min,max) are declared together with multi row actions : @1"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError ValuesQueryErrorWhileCloningCustumQfa
			 = new NeoDatis.Odb.Core.NeoDatisError(1046, "Error while cloning Query Field Action @1"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError ExecutionPlanIsNullQueryHasNotBeenExecuted
			 = new NeoDatis.Odb.Core.NeoDatisError(1047, "The query has not been executed yet so there is no execution plan available"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError ObjectWithOidDoesNotExist = 
			new NeoDatis.Odb.Core.NeoDatisError(1048, "Object with OID @1 does not exist in the database"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError ParamHelperWrongNoOfParams
			 = new NeoDatis.Odb.Core.NeoDatisError(1049, "The ParameterHelper for the class @1 didn't provide the correct number of parameters for the constructor @2"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError CacheIsFull = new NeoDatis.Odb.Core.NeoDatisError
			(1050, "Cache is full! ( it has @1 object(s). The maximum size is @2. Please increase the size of the cache using Configuration.setMaxNumberOfObjectInCache, or call the Configuration.setAutomaticallyIncreaseCacheSize(true)"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError UnregisteredBaseOnServer = 
			new NeoDatis.Odb.Core.NeoDatisError(1051, "Base @1 must be added on server before configuring it"
			);

		public static readonly NeoDatis.Odb.Core.IError UnsupportedEncoding = new NeoDatis.Odb.Core.NeoDatisError
			(1052, "Unsupported encoding @1");

		public static readonly NeoDatis.Odb.Core.IError ReconnectOnlyWithByteCodeAgentConfigured
			 = new NeoDatis.Odb.Core.NeoDatisError(1053, "Reconnect object only available when Byte code instrumentation is on"
			);

		public static readonly NeoDatis.Odb.Core.IError ReconnectOnlyForPreviouslyLoadedObject
			 = new NeoDatis.Odb.Core.NeoDatisError(1054, "Reconnect object only available for objets previously loaded in an ODB Session"
			);

		public static readonly NeoDatis.Odb.Core.IError ReconnectCanReconnectNullObject = 
			new NeoDatis.Odb.Core.NeoDatisError(1055, "Can not reconnect null object");

		public static readonly NeoDatis.Odb.Core.IError CanNotGetObjectFromNullOid = new 
			NeoDatis.Odb.Core.NeoDatisError(1056, "Can not get object from null OID");

		public static readonly NeoDatis.Odb.Core.IError InvalidOidRepresentation = new NeoDatis.Odb.Core.NeoDatisError
			(1057, "Invalid OID representation : @1");

		public static readonly NeoDatis.Odb.Core.IError DuplicatedKeyInIndex = new NeoDatis.Odb.Core.NeoDatisError
			(1058, "Duplicate key on index @1 : Values of index key @2");

		public static readonly NeoDatis.Odb.Core.IError OperationNotAllowedInTrigger = new 
			NeoDatis.Odb.Core.NeoDatisError(1056, "Operation not allowed in trigger");

		public static readonly NeoDatis.Odb.Core.IError CanNotAssociateServerTriggerToLocalOrClientOdb
			 = new NeoDatis.Odb.Core.NeoDatisError(1057, "Can not associate server trigger @1 to local or client ODB"
			);

		public static readonly NeoDatis.Odb.Core.IError TriggerCalledOnNullObject = new NeoDatis.Odb.Core.NeoDatisError
			(1058, "Trigger has been called on class @1 on a null object so it cannot retrieve the value of the '@2' attribute"
			);

		public static readonly NeoDatis.Odb.Core.IError CriteriaQueryOnUnknownObject = new 
			NeoDatis.Odb.Core.NeoDatisError(1059, "When the right side of a Criteria query is an object, this object must have been previously loaded by NeoDatis"
			);

		public static readonly NeoDatis.Odb.Core.IError ReconnectCanNotReconnectObject = 
			new NeoDatis.Odb.Core.NeoDatisError(1060, "Can not reconnect object");

		public static readonly NeoDatis.Odb.Core.NeoDatisError OdbCanNotDeleteNullObject = 
			new NeoDatis.Odb.Core.NeoDatisError(1061, "NeoDatis can not delete null object");

		public static readonly NeoDatis.Odb.Core.NeoDatisError FormtInvalidDateFormat = new 
			NeoDatis.Odb.Core.NeoDatisError(1062, "Invalid date format:@1, expecting something like @2"
			);

		public static readonly NeoDatis.Odb.Core.NeoDatisError InternalError = new NeoDatis.Odb.Core.NeoDatisError
			(10, "Internal error : @1 ");

		public NeoDatisError(int code, string description)
		{
			// Internal errors
			// *********************************************
			// User errors
			// *********************************************
			this.code = code;
			this.description = description;
		}

		public virtual NeoDatis.Odb.Core.IError AddParameter(object o)
		{
			if (parameters == null)
			{
				parameters = new NeoDatis.Tool.Wrappers.List.OdbArrayList<object>();
			}
			parameters.Add(o.ToString());
			return this;
		}

		public virtual NeoDatis.Odb.Core.IError AddParameter(string s)
		{
			if (parameters == null)
			{
				parameters = new NeoDatis.Tool.Wrappers.List.OdbArrayList<object>();
			}
			if (s == null)
			{
				parameters.Add("[null object]");
			}
			else
			{
				parameters.Add(s);
			}
			return this;
		}

		public virtual NeoDatis.Odb.Core.IError AddParameter(int i)
		{
			if (parameters == null)
			{
				parameters = new NeoDatis.Tool.Wrappers.List.OdbArrayList<object>();
			}
			parameters.Add(i);
			return this;
		}

		public virtual NeoDatis.Odb.Core.IError AddParameter(byte i)
		{
			if (parameters == null)
			{
				parameters = new NeoDatis.Tool.Wrappers.List.OdbArrayList<object>();
			}
			parameters.Add(i);
			return this;
		}

		public virtual NeoDatis.Odb.Core.IError AddParameter(long l)
		{
			if (parameters == null)
			{
				parameters = new NeoDatis.Tool.Wrappers.List.OdbArrayList<object>();
			}
			parameters.Add(l);
			return this;
		}

		/// <summary>replace the @1,@2,...</summary>
		/// <remarks>replace the @1,@2,... by their real values.</remarks>
		public override string ToString()
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			buffer.Append(code).Append(":").Append(description);
			string s = buffer.ToString();
			if (parameters != null)
			{
				for (int i = 0; i < parameters.Count; i++)
				{
					string parameterName = "@" + (i + 1);
					string parameterValue = parameters[i].ToString();
					int parameterIndex = s.IndexOf(parameterName);
					if (parameterIndex != -1)
					{
						s = NeoDatis.Tool.Wrappers.OdbString.ReplaceToken(s, parameterName, parameterValue
							, 1);
					}
				}
			}
			return s;
		}
	}
}
