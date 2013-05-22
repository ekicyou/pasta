namespace NeoDatis.Odb.Core
{
	/// <summary>All NeoDatis ODB Errors.</summary>
	/// <remarks>
	/// All NeoDatis ODB Errors. Errors can be user errors or Internal errors.
	/// All @1 in error description will be replaced by parameters
	/// </remarks>
	/// <1author>olivier s</1author>
	public class Error : NeoDatis.Odb.Core.IError
	{
		private int code;

		private string description;

		private NeoDatis.Tool.Wrappers.List.IOdbList<object> parameters;

		public static readonly NeoDatis.Odb.Core.Error NullNextObjectOid = new NeoDatis.Odb.Core.Error
			(100, "ODB has detected an inconsistency while reading instance(of @1) #@2 over @3 with oid @4 which has a null 'next object oid'"
			);

		public static readonly NeoDatis.Odb.Core.Error InstancePositionOutOfFile = new NeoDatis.Odb.Core.Error
			(101, "ODB is trying to read an instance at position @1 which is out of the file - File size is @2"
			);

		public static readonly NeoDatis.Odb.Core.Error InstancePositionIsNegative = new NeoDatis.Odb.Core.Error
			(102, "ODB is trying to read an instance at a negative position @1 , oid=@2 : @3"
			);

		public static readonly NeoDatis.Odb.Core.Error WrongTypeForBlockType = new NeoDatis.Odb.Core.Error
			(201, "Block type of wrong type : expected @1, Found @2 at position @3");

		public static readonly NeoDatis.Odb.Core.Error WrongBlockSize = new NeoDatis.Odb.Core.Error
			(202, "Wrong Block size : expected @1, Found @2 at position @3");

		public static readonly NeoDatis.Odb.Core.Error WrongOidAtPosition = new NeoDatis.Odb.Core.Error
			(203, "Reading object with oid @1 at position @2, but found oid @3");

		public static readonly NeoDatis.Odb.Core.Error BlockNumberDoesExist = new NeoDatis.Odb.Core.Error
			(205, "Block(of ids) with number @1 does not exist");

		public static readonly NeoDatis.Odb.Core.Error FoundPointer = new NeoDatis.Odb.Core.Error
			(204, "Found a pointer for oid @1 at position @2");

		public static readonly NeoDatis.Odb.Core.Error ObjectIsMarkedAsDeletedForOid = new 
			NeoDatis.Odb.Core.Error(206, "Object with oid @1 is marked as deleted");

		public static readonly NeoDatis.Odb.Core.Error ObjectIsMarkedAsDeletedForPosition
			 = new NeoDatis.Odb.Core.Error(207, "Object with position @1 is marked as deleted"
			);

		public static readonly NeoDatis.Odb.Core.Error NativeTypeNotSupported = new NeoDatis.Odb.Core.Error
			(208, "Native type not supported @1 @2");

		public static readonly NeoDatis.Odb.Core.Error NativeTypeDivergence = new NeoDatis.Odb.Core.Error
			(209, "Native type informed(@1) is different from the one informed (@2)");

		public static readonly NeoDatis.Odb.Core.Error NegativeClassNumberInHeader = new 
			NeoDatis.Odb.Core.Error(210, "number of classes is negative while reading database header : @1 at position @2"
			);

		public static readonly NeoDatis.Odb.Core.Error UnknownBlockType = new NeoDatis.Odb.Core.Error
			(211, "Unknown block type @1 at @2");

		public static readonly NeoDatis.Odb.Core.Error UnsupportedIoType = new NeoDatis.Odb.Core.Error
			(212, "Unsupported IO Type : @1");

		public static readonly NeoDatis.Odb.Core.Error ObjectDoesNotExistInCache = new NeoDatis.Odb.Core.Error
			(213, "Object does not exist in cache");

		public static readonly NeoDatis.Odb.Core.Error ObjectWithOidDoesNotExistInCache = 
			new NeoDatis.Odb.Core.Error(213, "Object with oid @1 does not exist in cache");

		public static readonly NeoDatis.Odb.Core.Error ObjectInfoNotInTempCache = new NeoDatis.Odb.Core.Error
			(214, "ObjectInfo does not exist in temporary cache oid=@1 and position=@2");

		public static readonly NeoDatis.Odb.Core.Error CanNotDeleteFile = new NeoDatis.Odb.Core.Error
			(215, "Can not delete file @1");

		public static readonly NeoDatis.Odb.Core.Error GoToPosition = new NeoDatis.Odb.Core.Error
			(216, "Error while going to position @1, length = @2");

		public static readonly NeoDatis.Odb.Core.Error ErrorInCoreProviderInitialization = 
			new NeoDatis.Odb.Core.Error(217, "Error while initializing CoreProvider @1");

		public static readonly NeoDatis.Odb.Core.Error UndefinedClassInfo = new NeoDatis.Odb.Core.Error
			(218, "Undefined class info for @1");

		public static readonly NeoDatis.Odb.Core.Error AbstractObjectInfoTypeNotSupported
			 = new NeoDatis.Odb.Core.Error(219, "Abstract Object Info type not supported : @1"
			);

		public static readonly NeoDatis.Odb.Core.Error NegativeBlockSize = new NeoDatis.Odb.Core.Error
			(220, "Negative block size at @1 : size = @2, object=@3");

		public static readonly NeoDatis.Odb.Core.Error InplaceUpdateNotPossibleForArray = 
			new NeoDatis.Odb.Core.Error(221, "Array in place update with array smaller than element array index to update : array size=@1, element index=@2"
			);

		public static readonly NeoDatis.Odb.Core.Error OperationNotImplemented = new NeoDatis.Odb.Core.Error
			(222, "Operation not supported : @1");

		public static readonly NeoDatis.Odb.Core.Error InstanceBuilderWrongObjectType = new 
			NeoDatis.Odb.Core.Error(223, "Wrong type of object: expecting @1 and received @2"
			);

		public static readonly NeoDatis.Odb.Core.Error InstanceBuilderWrongObjectContainerType
			 = new NeoDatis.Odb.Core.Error(224, "Building instance of @1 : can not put a @2 into a @3"
			);

		public static readonly NeoDatis.Odb.Core.Error InstanceBuilderNativeTypeInCollectionNotSupported
			 = new NeoDatis.Odb.Core.Error(225, "Native @1 in Collection(List,array,Map) not supported"
			);

		public static readonly NeoDatis.Odb.Core.Error ObjectIntrospectorNoFieldWithName = 
			new NeoDatis.Odb.Core.Error(226, "Class/Interface @1 does not have attribute '@2'"
			);

		public static readonly NeoDatis.Odb.Core.Error ObjectIntrospectorClassNotFound = 
			new NeoDatis.Odb.Core.Error(227, "Class not found : @1");

		public static readonly NeoDatis.Odb.Core.Error ClassPoolCreateClass = new NeoDatis.Odb.Core.Error
			(228, "Error while creating (reflection) class @1");

		public static readonly NeoDatis.Odb.Core.Error BufferTooSmall = new NeoDatis.Odb.Core.Error
			(229, "Buffer too small: buffer size = @1 and data size = @2 - should not happen"
			);

		public static readonly NeoDatis.Odb.Core.Error FileInterfaceWriteBytesNotImplementedForTransaction
			 = new NeoDatis.Odb.Core.Error(230, "writeBytes not implemented for transactions"
			);

		public static readonly NeoDatis.Odb.Core.Error FileInterfaceReadError = new NeoDatis.Odb.Core.Error
			(231, "Error reading @1 bytes at @2 : read @3 bytes instead");

		public static readonly NeoDatis.Odb.Core.Error PointerToSelf = new NeoDatis.Odb.Core.Error
			(232, "Error while creating a pointer : a pointer to itself : @1 -> @2 for oid @3"
			);

		public static readonly NeoDatis.Odb.Core.Error IndexNotFound = new NeoDatis.Odb.Core.Error
			(233, "No index defined on class @1 at index position @2");

		public static readonly NeoDatis.Odb.Core.Error NotYetImplemented = new NeoDatis.Odb.Core.Error
			(234, "Not yet implemented : @1");

		public static readonly NeoDatis.Odb.Core.Error MetaModelClassNameDoesNotExist = new 
			NeoDatis.Odb.Core.Error(235, "Class @1 does not exist in meta-model");

		public static readonly NeoDatis.Odb.Core.Error MetaModelClassWithOidDoesNotExist = 
			new NeoDatis.Odb.Core.Error(236, "Class with oid @1 does not exist in meta-model"
			);

		public static readonly NeoDatis.Odb.Core.Error MetaModelClassWithPositionDoesNotExist
			 = new NeoDatis.Odb.Core.Error(237, "Class with position @1 does not exist in meta-model"
			);

		public static readonly NeoDatis.Odb.Core.Error ClassInfoDoNotHaveTheAttribute = new 
			NeoDatis.Odb.Core.Error(238, "Class @1 does not have attribute with name @2");

		public static readonly NeoDatis.Odb.Core.Error OdbTypeIdDoesNotExist = new NeoDatis.Odb.Core.Error
			(239, "ODBtype with id @1 does not exist");

		public static readonly NeoDatis.Odb.Core.Error OdbTypeNativeTypeWithIdDoesNotExist
			 = new NeoDatis.Odb.Core.Error(240, "Native type with id @1 does not exist");

		public static readonly NeoDatis.Odb.Core.Error QueryEngineNotSet = new NeoDatis.Odb.Core.Error
			(241, "Storage engine not set on query");

		public static readonly NeoDatis.Odb.Core.Error QueryTypeNotImplemented = new NeoDatis.Odb.Core.Error
			(242, "Query type @1 not implemented");

		public static readonly NeoDatis.Odb.Core.Error CryptoAlgorithNotFound = new NeoDatis.Odb.Core.Error
			(243, "Could not get the MD5 algorithm to encrypt the password");

		public static readonly NeoDatis.Odb.Core.Error XmlHeader = new NeoDatis.Odb.Core.Error
			(244, "Error while creating XML Header");

		public static readonly NeoDatis.Odb.Core.Error XmlReservingIds = new NeoDatis.Odb.Core.Error
			(245, "Error while reserving @1 ids");

		public static readonly NeoDatis.Odb.Core.Error XmlSettingMetaModel = new NeoDatis.Odb.Core.Error
			(246, "Error while setting meta model");

		public static readonly NeoDatis.Odb.Core.Error SerializationFromString = new NeoDatis.Odb.Core.Error
			(247, "Error while deserializing: expecting classId @1 and received @2");

		public static readonly NeoDatis.Odb.Core.Error SerializationCollection = new NeoDatis.Odb.Core.Error
			(248, "Error while deserializing collection: sizes are not consistent : expected @1, found @2"
			);

		public static readonly NeoDatis.Odb.Core.Error MetamodelReadingLastObject = new NeoDatis.Odb.Core.Error
			(249, "Error while reading last object of type @1 at with OID @2");

		public static readonly NeoDatis.Odb.Core.Error CacheNegativeOid = new NeoDatis.Odb.Core.Error
			(250, "Negative oid set in cache @1");

		public static readonly NeoDatis.Odb.Core.Error ClientServerSynchronizeIds = new NeoDatis.Odb.Core.Error
			(251, "Error while synchronizing oids,length are <>, local=@1, client=@2");

		public static readonly NeoDatis.Odb.Core.Error ClientServerCanNotOpenOdbServerOnPort
			 = new NeoDatis.Odb.Core.Error(252, "Can not start ODB server on port @1");

		public static readonly NeoDatis.Odb.Core.Error ClientServerCanNotAssociateOids = 
			new NeoDatis.Odb.Core.Error(253, "Can not associate server and client oids : server oid=@1 and client oid=@2"
			);

		public static readonly NeoDatis.Odb.Core.Error SessionDoesNotExistForThread = new 
			NeoDatis.Odb.Core.Error(254, "Thread @1 for base @2 does not have any associated session, base id=@3"
			);

		public static readonly NeoDatis.Odb.Core.Error ClientServerUnknownCommand = new NeoDatis.Odb.Core.Error
			(255, "Unknown server command : @1");

		public static readonly NeoDatis.Odb.Core.Error ClientServerError = new NeoDatis.Odb.Core.Error
			(256, "ServerSide Error : @1");

		public static readonly NeoDatis.Odb.Core.Error ObjectReaderDirectCall = new NeoDatis.Odb.Core.Error
			(257, "Generic readObjectInfo called for non native object info");

		public static readonly NeoDatis.Odb.Core.Error CacheObjectInfoHeaderWithoutClassId
			 = new NeoDatis.Odb.Core.Error(258, "Object Info Header without class id ; oih.oid=@1"
			);

		public static readonly NeoDatis.Odb.Core.Error NonNativeAttributeStoredByPositionInsteadOfOid
			 = new NeoDatis.Odb.Core.Error(259, "Non native attribute (@1) of class @2 stored by position @3 instead of oid"
			);

		public static readonly NeoDatis.Odb.Core.Error CacheNullOid = new NeoDatis.Odb.Core.Error
			(260, "Null OID");

		public static readonly NeoDatis.Odb.Core.Error NegativePosition = new NeoDatis.Odb.Core.Error
			(261, "Negative position : @1");

		public static readonly NeoDatis.Odb.Core.Error UnexpectedSituation = new NeoDatis.Odb.Core.Error
			(262, "Unexpected situation: @1");

		public static readonly NeoDatis.Odb.Core.Error ImportError = new NeoDatis.Odb.Core.Error
			(263, "Import error: @1");

		public static readonly NeoDatis.Odb.Core.Error ClientServerCanNotCreateClassInfo = 
			new NeoDatis.Odb.Core.Error(264, "ServerSide Error : Can not create class info @1"
			);

		public static readonly NeoDatis.Odb.Core.Error ClientServerMetaModelInconsistency
			 = new NeoDatis.Odb.Core.Error(265, "ServerSide Error : Meta model on server and client are inconsistent : class @1 exist on server and does not exist on the client!"
			);

		public static readonly NeoDatis.Odb.Core.Error ClientServerMetaModelInconsistencyDifferentOid
			 = new NeoDatis.Odb.Core.Error(266, "ServerSide Error : Meta model on server and client are inconsistent : class @1 have different OIDs on server (@2) and client(@3)!"
			);

		public static readonly NeoDatis.Odb.Core.Error MethodShouldNotBeCalled = new NeoDatis.Odb.Core.Error
			(267, "Method @1 should not be called on @2");

		public static readonly NeoDatis.Odb.Core.Error CacheNegativePosition = new NeoDatis.Odb.Core.Error
			(268, "Caching an ObjectInfoHeader with negative position @1");

		public static readonly NeoDatis.Odb.Core.Error ErrorWhileGettingObjectFromListAtIndex
			 = new NeoDatis.Odb.Core.Error(269, "Error while getting object from list at index @1"
			);

		public static readonly NeoDatis.Odb.Core.Error ClassInfoDoesNotExistInMetaModel = 
			new NeoDatis.Odb.Core.Error(270, "Class Info @1 does not exist in MetaModel");

		public static readonly NeoDatis.Odb.Core.Error BtreeSizeDiffersFromClassElementNumber
			 = new NeoDatis.Odb.Core.Error(271, "The Index has @1 element(s) whereas the Class has @2 objects. The two values should be equal"
			);

		public static readonly NeoDatis.Odb.Core.Error ClientServerConnectionIsNull = new 
			NeoDatis.Odb.Core.Error(272, "The connection ID @1 does not exist in connection manager (@2)"
			);

		public static readonly NeoDatis.Odb.Core.Error ClientServerPortIsBusy = new NeoDatis.Odb.Core.Error
			(273, "Can not start ODB server on port @1: The port is busy. Check if another server is not already running of this port"
			);

		public static readonly NeoDatis.Odb.Core.Error InstanceBuilderNativeType = new NeoDatis.Odb.Core.Error
			(274, "Native object of type @1 can not be instanciated");

		public static readonly NeoDatis.Odb.Core.Error ClassIntrospectionError = new NeoDatis.Odb.Core.Error
			(275, "Class Introspectpr error for class @1");

		public static readonly NeoDatis.Odb.Core.IError EndOfFileReached = new NeoDatis.Odb.Core.Error
			(276, "End Of File reached - position = @1 : Length = @2");

		public static readonly NeoDatis.Odb.Core.IError MapInstanciationError = new NeoDatis.Odb.Core.Error
			(277, "Error while creating instance of MAP of class @1");

		public static readonly NeoDatis.Odb.Core.IError CollectionInstanciationError = new 
			NeoDatis.Odb.Core.Error(278, "Error while creating instance of Collection of class @1"
			);

		public static readonly NeoDatis.Odb.Core.IError InstanciationError = new NeoDatis.Odb.Core.Error
			(279, "Error while creating instance of type @1");

		public static readonly NeoDatis.Odb.Core.IError ServerSideError = new NeoDatis.Odb.Core.Error
			(280, "Server side error @1 : @2");

		public static readonly NeoDatis.Odb.Core.IError NetSerialisationError = new NeoDatis.Odb.Core.Error
			(281, "Net Serialization Error");

		public static readonly NeoDatis.Odb.Core.IError ClientNetError = new NeoDatis.Odb.Core.Error
			(282, "Client Net Error");

		public static readonly NeoDatis.Odb.Core.IError ServerNetError = new NeoDatis.Odb.Core.Error
			(283, "Server Net Error");

		public static readonly NeoDatis.Odb.Core.IError ErrorWhileGettingConstrctorsOfClass
			 = new NeoDatis.Odb.Core.Error(284, "Error while getting constructor of @1");

		public static readonly NeoDatis.Odb.Core.IError UnknownHost = new NeoDatis.Odb.Core.Error
			(285, "Unknown host");

		public static readonly NeoDatis.Odb.Core.Error CacheNullObject = new NeoDatis.Odb.Core.Error
			(286, "Null Object : @1");

		public static readonly NeoDatis.Odb.Core.Error LookupKeyNotFound = new NeoDatis.Odb.Core.Error
			(287, "Lookup key not found : @1");

		public static readonly NeoDatis.Odb.Core.Error ServerError = new NeoDatis.Odb.Core.Error
			(288, "Server error : @1");

		public static readonly NeoDatis.Odb.Core.Error ReflectionErrorWhileGettingField = 
			new NeoDatis.Odb.Core.Error(289, "Error while getting field @1 on class @2");

		public static readonly NeoDatis.Odb.Core.Error NotYetSupported = new NeoDatis.Odb.Core.Error
			(290, "Not Yet Supported : @1");

		public static readonly NeoDatis.Odb.Core.Error FileNotFound = new NeoDatis.Odb.Core.Error
			(291, "File not found: @1");

		public static readonly NeoDatis.Odb.Core.Error CriteriaQueryUnknownAttribute = new 
			NeoDatis.Odb.Core.Error(1000, "Attribute @1 used in criteria queria does not exist on class @2"
			);

		public static readonly NeoDatis.Odb.Core.Error RuntimeIncompatibleVersion = new NeoDatis.Odb.Core.Error
			(1001, "Incompatible ODB Version : ODB file version is @1 and Runtime version is @2"
			);

		public static readonly NeoDatis.Odb.Core.Error IncompatibleMetamodel = new NeoDatis.Odb.Core.Error
			(1002, "Incompatible meta-model : @1");

		public static readonly NeoDatis.Odb.Core.Error IncompatibleJavaVm = new NeoDatis.Odb.Core.Error
			(1003, "Incompatible java virtual Machine, 1.5 or greater is required, you are using : @1"
			);

		public static readonly NeoDatis.Odb.Core.Error OdbIsClosed = new NeoDatis.Odb.Core.Error
			(1004, "ODB session has already been closed (@1)");

		public static readonly NeoDatis.Odb.Core.Error OdbHasBeenRollbacked = new NeoDatis.Odb.Core.Error
			(1005, "ODB session has been rollbacked (@1)");

		public static readonly NeoDatis.Odb.Core.Error OdbCanNotStoreNullObject = new NeoDatis.Odb.Core.Error
			(1006, "ODB can not store null object");

		public static readonly NeoDatis.Odb.Core.Error OdbCanNotStoreArrayDirectly = new 
			NeoDatis.Odb.Core.Error(1007, "ODB can not store array directly : @1");

		public static readonly NeoDatis.Odb.Core.Error OdbCanNotStoreNativeObjectDirectly
			 = new NeoDatis.Odb.Core.Error(1008, "ODB can not store native object direclty : @1"
			);

		public static readonly NeoDatis.Odb.Core.Error ObjectDoesNotExistInCacheForDelete
			 = new NeoDatis.Odb.Core.Error(1009, "The object being deleted does not exist in cache. Make sure the object has been loaded before deleting : @1"
			);

		public static readonly NeoDatis.Odb.Core.Error TransactionIsPending = new NeoDatis.Odb.Core.Error
			(1010, "There are pending work associated to current transaction, a commit or rollback should be executed : session id = @1"
			);

		public static readonly NeoDatis.Odb.Core.Error UnknownObjectToGetOid = new NeoDatis.Odb.Core.Error
			(1011, "Unknown object @1");

		public static readonly NeoDatis.Odb.Core.Error OdbCanNotReturnOidOfNullObject = new 
			NeoDatis.Odb.Core.Error(1012, "Can not return the oid of a null object");

		public static readonly NeoDatis.Odb.Core.Error OdbFileIsLockedByCurrentVirtualMachine
			 = new NeoDatis.Odb.Core.Error(1013, "@1 file is locked by the current Virtual machine - check if the database has not been opened in the current VM! : thread = @2 - using multi thread ? @3"
			);

		public static readonly NeoDatis.Odb.Core.Error OdbFileIsLockedByExternalProgram = 
			new NeoDatis.Odb.Core.Error(1014, "@1 file is locked - check if the database file is not opened in another program! : thread = @2 - using multi thread ? @3"
			);

		public static readonly NeoDatis.Odb.Core.Error UserNameTooLong = new NeoDatis.Odb.Core.Error
			(1015, "User name @1 is too long, should be lesser than 20 characters");

		public static readonly NeoDatis.Odb.Core.Error PasswordTooLong = new NeoDatis.Odb.Core.Error
			(1016, "Password is too long, it must be less than 20 character long");

		public static readonly NeoDatis.Odb.Core.Error TransactionAlreadyCommitedOrRollbacked
			 = new NeoDatis.Odb.Core.Error(1017, "Transaction have already been 'committed' or 'rollbacked'"
			);

		public static readonly NeoDatis.Odb.Core.Error DifferentSizeInWriteAction = new NeoDatis.Odb.Core.Error
			(1018, "Size difference in WriteAction.persist :(calculated,stored)=(@1,@2)");

		public static readonly NeoDatis.Odb.Core.Error ClassWithoutConstructor = new NeoDatis.Odb.Core.Error
			(1019, "Class without any constructor : @1");

		public static readonly NeoDatis.Odb.Core.Error NoNullableConstructor = new NeoDatis.Odb.Core.Error
			(1020, "Constructor @1 of class @2 was called with null values because it does not have default constructor and it seems the constructor is not prepared for this!"
			);

		public static readonly NeoDatis.Odb.Core.Error QueryBadCriteria = new NeoDatis.Odb.Core.Error
			(1021, "CollectionSizeCriteria only work with Collection or Array, and you passed a @1 instead"
			);

		public static readonly NeoDatis.Odb.Core.Error QueryCollectionSizeCriteriaNotSupported
			 = new NeoDatis.Odb.Core.Error(1022, "CollectionSizeCriterion sizeType @1 not yet implemented"
			);

		public static readonly NeoDatis.Odb.Core.Error QueryComparableCriteriaAppliedOnNonComparable
			 = new NeoDatis.Odb.Core.Error(1023, "ComparisonCriteria with greater than only work with Comparable, and you passed a @1 instead"
			);

		public static readonly NeoDatis.Odb.Core.Error QueryUnknownOperator = new NeoDatis.Odb.Core.Error
			(1024, "Unknow operator @1");

		public static readonly NeoDatis.Odb.Core.Error QueryContainsCriterionTypeNotSupported
			 = new NeoDatis.Odb.Core.Error(1025, "ContainsCriterion: type @1 not supported!"
			);

		public static readonly NeoDatis.Odb.Core.Error QueryAttributeTypeNotSupportedInLikeExpression
			 = new NeoDatis.Odb.Core.Error(1026, "LikeCriteria with like expression(%) only work with String, and you passed a @1 instead"
			);

		public static readonly NeoDatis.Odb.Core.Error IndexKeysMustImplementComparable = 
			new NeoDatis.Odb.Core.Error(1027, "Unable to build index key for attribute that does not implement 'Comparable/IComparable' : Index=@1, attribute = @2 , type = @3"
			);

		public static readonly NeoDatis.Odb.Core.Error QueryNqMatchMethodNotImplemented = 
			new NeoDatis.Odb.Core.Error(1029, "ISimpleNativeQuery implementing classes must implement method: boolean match(?Object obj), class @1 does not"
			);

		public static readonly NeoDatis.Odb.Core.Error QueryNqExceptionRaisedByNativeQueryExecution
			 = new NeoDatis.Odb.Core.Error(1030, "Exception raised by the native query @1 match method"
			);

		public static readonly NeoDatis.Odb.Core.Error OdbCanNotReturnOidOfUnknownObject = 
			new NeoDatis.Odb.Core.Error(1031, "Can not return the oid of a not previously loaded object : @1"
			);

		public static readonly NeoDatis.Odb.Core.Error ErrorWhileAddingObjectToHashmap = 
			new NeoDatis.Odb.Core.Error(1032, "Internal error in user object of class @1 in equals or hashCode method : @2"
			);

		public static readonly NeoDatis.Odb.Core.Error AttributeReferencesADeletedObject = 
			new NeoDatis.Odb.Core.Error(1033, "Object of type @1 with oid @2 has the attribute '@3' that references a deleted object"
			);

		public static readonly NeoDatis.Odb.Core.Error BeforeDeleteTriggerHasThrownException
			 = new NeoDatis.Odb.Core.Error(1034, "Before Delete Trigger @1 has thrown exception. ODB has ignored it \n<user exception>\n@2</user exception>"
			);

		public static readonly NeoDatis.Odb.Core.Error AfterDeleteTriggerHasThrownException
			 = new NeoDatis.Odb.Core.Error(1035, "After Delete Trigger @1 has thrown exception. ODB has ignored it\n<user exception>\n@2</user exception>"
			);

		public static readonly NeoDatis.Odb.Core.Error BeforeUpdateTriggerHasThrownException
			 = new NeoDatis.Odb.Core.Error(1036, "Before Update Trigger @1 has thrown exception. ODB has ignored it\n<user exception>\n@2</user exception>"
			);

		public static readonly NeoDatis.Odb.Core.Error AfterUpdateTriggerHasThrownException
			 = new NeoDatis.Odb.Core.Error(1037, "After Update Trigger @1 has thrown exception. ODB has ignored it\n<user exception>\n@2</user exception>"
			);

		public static readonly NeoDatis.Odb.Core.Error BeforeInsertTriggerHasThrownException
			 = new NeoDatis.Odb.Core.Error(1038, "Before Insert Trigger @1 has thrown exception. ODB has ignored it\n<user exception>\n@2</user exception>"
			);

		public static readonly NeoDatis.Odb.Core.Error AfterInsertTriggerHasThrownException
			 = new NeoDatis.Odb.Core.Error(1039, "After Insert Trigger @1 has thrown exception. ODB has ignored it\n<user exception>\n@2</user exception>"
			);

		public static readonly NeoDatis.Odb.Core.Error NoMoreObjectsInCollection = new NeoDatis.Odb.Core.Error
			(1040, "No more objects in collection");

		public static readonly NeoDatis.Odb.Core.Error IndexAlreadyExist = new NeoDatis.Odb.Core.Error
			(1041, "Index @1 already exist on class @2");

		public static readonly NeoDatis.Odb.Core.Error IndexDoesNotExist = new NeoDatis.Odb.Core.Error
			(1042, "Index @1 does not exist on class @2");

		public static readonly NeoDatis.Odb.Core.Error QueryAttributeTypeNotSupportedInIequalExpression
			 = new NeoDatis.Odb.Core.Error(1043, "EqualCriteria with case insensitive expression only work with String, and you passed a @1 instead"
			);

		public static readonly NeoDatis.Odb.Core.Error ValuesQueryAliasDoesNotExist = new 
			NeoDatis.Odb.Core.Error(1044, "Alias @1 does not exist in query result. Existing alias are @2"
			);

		public static readonly NeoDatis.Odb.Core.Error ValuesQueryNotConsistent = new NeoDatis.Odb.Core.Error
			(1045, "Single row actions (like sum,count,min,max) are declared together with multi row actions : @1"
			);

		public static readonly NeoDatis.Odb.Core.Error ValuesQueryErrorWhileCloningCustumQfa
			 = new NeoDatis.Odb.Core.Error(1046, "Error while cloning Query Field Action @1"
			);

		public static readonly NeoDatis.Odb.Core.Error ExecutionPlanIsNullQueryHasNotBeenExecuted
			 = new NeoDatis.Odb.Core.Error(1047, "The query has not been executed yet so there is no execution plan available"
			);

		public static readonly NeoDatis.Odb.Core.Error ObjectWithOidDoesNotExist = new NeoDatis.Odb.Core.Error
			(1048, "Object with OID @1 does not exist in the database");

		public static readonly NeoDatis.Odb.Core.Error ParamHelperWrongNoOfParams = new NeoDatis.Odb.Core.Error
			(1049, "The ParameterHelper for the class @1 didn't provide the correct number of parameters for the constructor @2"
			);

		public static readonly NeoDatis.Odb.Core.Error CacheIsFull = new NeoDatis.Odb.Core.Error
			(1050, "Cache is full! ( it has @1 object(s). The maximum size is @2. Please increase the size of the cache using Configuration.setMaxNumberOfObjectInCache, or call the Configuration.setAutomaticallyIncreaseCacheSize(true)"
			);

		public static readonly NeoDatis.Odb.Core.Error UnregisteredBaseOnServer = new NeoDatis.Odb.Core.Error
			(1051, "Base @1 must be added on server before configuring it");

		public static readonly NeoDatis.Odb.Core.IError UnsupportedEncoding = new NeoDatis.Odb.Core.Error
			(1052, "Unsupported encoding @1");

		public static readonly NeoDatis.Odb.Core.IError ReconnectOnlyWithByteCodeAgentConfigured
			 = new NeoDatis.Odb.Core.Error(1053, "Reconnect object only available when Byte code instrumentation is on"
			);

		public static readonly NeoDatis.Odb.Core.IError ReconnectOnlyForPreviouslyLoadedObject
			 = new NeoDatis.Odb.Core.Error(1054, "Reconnect object only available for objets previously loaded in an ODB Session"
			);

		public static readonly NeoDatis.Odb.Core.IError ReconnectCanReconnectNullObject = 
			new NeoDatis.Odb.Core.Error(1055, "Can not reconnect null object");

		public static readonly NeoDatis.Odb.Core.IError CanNotGetObjectFromNullOid = new 
			NeoDatis.Odb.Core.Error(1056, "Can not get object from null OID");

		public static readonly NeoDatis.Odb.Core.IError InvalidOidRepresentation = new NeoDatis.Odb.Core.Error
			(1057, "Invalid OID representation : @1");

		public static readonly NeoDatis.Odb.Core.IError DuplicatedKeyInIndex = new NeoDatis.Odb.Core.Error
			(1058, "Duplicate key on index @1 : Values of index key @2");

		public static readonly NeoDatis.Odb.Core.IError OperationNotAllowedInTrigger = new 
			NeoDatis.Odb.Core.Error(1056, "Operation not allowed in trigger");

		public static readonly NeoDatis.Odb.Core.Error InternalError = new NeoDatis.Odb.Core.Error
			(10, "Internal error : @1 ");

		public Error(int code, string description)
		{
			// Internal errors
			// User errors
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
			parameters.Add(s);
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
