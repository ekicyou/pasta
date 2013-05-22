namespace NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine
{
	/// <summary>Manage all IO writing</summary>
	/// <author>olivier s</author>
	public class ObjectWriterNew
	{
		//	protected static int nbInPlaceUpdates = 0;
		//
		//	protected static int nbNormalUpdates = 0;
		//
		//	public static final String LOG_ID = "ObjectWriter";
		//
		//	private static final String LOG_ID_DEBUG = "ObjectWriter.debug";
		//
		//	protected IStorageEngine storageEngine;
		//
		//	protected IObjectReader objectReader;
		//
		//	private ISession session;
		//
		//	private IFileSystemInterface fsi;
		//
		//	// Just for display matters
		//	private int currentDepth;
		//
		//	protected IIdManager idManager;
		//
		//	/** To manage triggers */
		//	private ITriggerManager triggerManager;
		//	
		//	private IByteArrayConverter byteArrayConverter;
		//
		//	private static int nbCallsToUpdate;
		//
		//	static final private int NON_NATIVE_HEADER_BLOCK_SIZE = ODBType.INTEGER.getSize() + ODBType.BYTE.getSize() + ODBType.LONG.getSize();
		//
		//	static final private int NATIVE_HEADER_BLOCK_SIZE = ODBType.INTEGER.getSize() + ODBType.BYTE.getSize() + ODBType.INTEGER.getSize();
		//
		//	private boolean isLocalMode;
		//
		//	public ObjectWriterNew(IStorageEngine engine, ITriggerManager triggerManager) {
		//		this.storageEngine = engine;
		//		this.objectReader = storageEngine.getObjectReader();
		//		this.session = engine.getSession(true);
		//		this.fsi = engine.getObjectWriter().getFsi();
		//		this.triggerManager = triggerManager;
		//		this.isLocalMode = this.storageEngine.isLocal();
		//		this.byteArrayConverter = Configuration.getCoreProvider().getByteArrayConverter();
		//	}
		//
		//	public void initIdManager() {
		//		/*
		//		this.idManager = new IdManager(this, objectReader, storageEngine.getCurrentIdBlockPosition(), storageEngine
		//				.getCurrentIdBlockNumber(), storageEngine.getCurrentIdBlockMaxOid());
		//				*/
		//	}
		//
		//	/**
		//	 * Creates the header of the file
		//	 * 
		//	 * @param creationDate
		//	 *            The creation date
		//	 * @param user
		//	 *            The user
		//	 * @param password
		//	 *            The password
		//	 * 
		//	 * @throws IOException
		//	 */
		//	public void createEmptyDatabaseHeader(long creationDate, String user, String password) throws IOException {
		//		writeVersion(false);
		//		writeDatabaseId(creationDate, false);
		//		writeNumberOfClasses(0, false);
		//		writeFirstClassInfoOID(StorageEngineConstant.NULL_OBJECT_ID, false);
		//		writeLastODBCloseStatus(false, false);
		//		writeUserAndPassword(user, password, false);
		//
		//		// This is the position of the first block id. But it will always
		//		// contain the position of the current id block
		//		fsi.writeLong(StorageEngineConstant.DATABASE_HEADER_FIRST_ID_BLOCK_POSITION, false, "current id block position",
		//				DefaultWriteAction.DIRECT_WRITE_ACTION);
		//		// Write an empty id block
		//		writeIdBlock(-1, Configuration.getIdBlockSize(), BlockStatus.BLOCK_NOT_FULL, 1, -1, false);
		//
		//		flush();
		//
		//		storageEngine
		//				.setCurrentIdBlockInfos(StorageEngineConstant.DATABASE_HEADER_FIRST_ID_BLOCK_POSITION, 1, OIDFactory.buildObjectOID(0));
		//	}
		//
		//	private void writeUserAndPassword(String user, String password, boolean writeInTransaction) throws IOException {
		//		if (user != null && password != null) {
		//			String encryptedPassword = Cryptographer.encrypt(password);
		//			fsi.writeBoolean(true, writeInTransaction, "has user and password");
		//			if (user.length() > 20) {
		//				throw new ODBRuntimeException(Error.USER_NAME_TOO_LONG.addParameter(user).addParameter(20));
		//			}
		//			if (password.length() > 20) {
		//				throw new ODBRuntimeException(Error.PASSWORD_TOO_LONG.addParameter(20));
		//			}
		//			fsi.writeString(user, writeInTransaction, 50);
		//			fsi.setWritePosition(StorageEngineConstant.DATABASE_HEADER_DATABASE_PASSWORD, writeInTransaction);
		//			fsi.writeString(encryptedPassword, writeInTransaction, 50);
		//		} else {
		//			fsi.writeBoolean(false, writeInTransaction, "database without user and password");
		//			fsi.writeString("no-user", writeInTransaction, 50);
		//			fsi.writeString("no-password", writeInTransaction, 50);
		//		}
		//	}
		//
		//	/** Write the version in the database file */
		//	protected void writeVersion(boolean writeInTransaction) throws IOException {
		//		fsi.setWritePosition(StorageEngineConstant.DATABASE_HEADER_VERSION_POSITION, writeInTransaction);
		//		fsi.writeInt(StorageEngineConstant.CURRENT_FILE_FORMAT_VERSION, writeInTransaction, "database file format version");
		//		storageEngine.setVersion(StorageEngineConstant.CURRENT_FILE_FORMAT_VERSION);
		//	}
		//
		//	private void writeDatabaseId(long creationDate, boolean writeInTransaction) throws IOException {
		//		long[] id = UUID.getDatabaseId(creationDate);
		//		fsi.writeLong(id[0], writeInTransaction, "database id 1/4", DefaultWriteAction.DIRECT_WRITE_ACTION);
		//		fsi.writeLong(id[1], writeInTransaction, "database id 2/4", DefaultWriteAction.DIRECT_WRITE_ACTION);
		//		fsi.writeLong(id[2], writeInTransaction, "database id 3/4", DefaultWriteAction.DIRECT_WRITE_ACTION);
		//		fsi.writeLong(id[3], writeInTransaction, "database id 4/4", DefaultWriteAction.DIRECT_WRITE_ACTION);
		//		storageEngine.setDatabaseId(id);
		//	}
		//
		//	/** Write the number of classes in metamodel */
		//	protected void writeNumberOfClasses(int number, boolean writeInTransaction) throws IOException {
		//		fsi.setWritePosition(StorageEngineConstant.DATABASE_HEADER_NUMBER_OF_CLASSES_POSITION, writeInTransaction);
		//		fsi.writeInt(number, writeInTransaction, "nb classes");
		//	}
		//
		//	/** Write the status of the last odb close */
		//	protected void writeLastODBCloseStatus(boolean ok, boolean writeInTransaction) throws IOException {
		//		fsi.setWritePosition(StorageEngineConstant.DATABASE_HEADER_LAST_CLOSE_STATUS_POSITION, writeInTransaction);
		//		fsi.writeBoolean(ok, writeInTransaction, "odb last close status");
		//	}
		//
		//	/**
		//	 * Writes the header of a block of type ID - a block that contains ids of
		//	 * objects and classes
		//	 * 
		//	 * @param position
		//	 *            Positoion at which the block must be written, if -1, take the
		//	 *            next available position
		//	 * @param idBlockSize
		//	 *            The block size in byte
		//	 * @param blockStatus
		//	 *            The block status
		//	 * @param blockNumber
		//	 *            The number of the block
		//	 * @param previousBlockPosition
		//	 *            The position of the previous block of the same type
		//	 * @param writeInTransaction
		//	 *            To indicate if write must be done in transaction
		//	 * @return The position of the id
		//	 * @throws IOException
		//	 */
		//	public long writeIdBlock(long position, int idBlockSize, byte blockStatus, int blockNumber, long previousBlockPosition,
		//			boolean writeInTransaction) throws IOException {
		//		if (position == -1) {
		//			position = fsi.getAvailablePosition();
		//		}
		//		// LogUtil.fileSystemOn(true);
		//		// Updates the database header with the current id block position
		//		fsi.setWritePosition(StorageEngineConstant.DATABASE_HEADER_CURRENT_ID_BLOCK_POSITION, writeInTransaction);
		//		fsi.writeLong(position, false, "current id block position", DefaultWriteAction.DIRECT_WRITE_ACTION);
		//
		//		fsi.setWritePosition(position, writeInTransaction);
		//
		//		fsi.writeInt(idBlockSize, writeInTransaction, "block size");
		//		// LogUtil.fileSystemOn(false);
		//		fsi.writeByte(BlockTypes.BLOCK_TYPE_IDS, writeInTransaction);
		//		fsi.writeByte(blockStatus, writeInTransaction);
		//		// prev position
		//		fsi.writeLong(previousBlockPosition, writeInTransaction, "prev block pos", DefaultWriteAction.DIRECT_WRITE_ACTION);
		//		// next position
		//		fsi.writeLong(-1, writeInTransaction, "next block pos", DefaultWriteAction.DIRECT_WRITE_ACTION);
		//		fsi.writeInt(blockNumber, writeInTransaction, "id block number");
		//		fsi.writeLong(0, writeInTransaction, "id block max id", DefaultWriteAction.DIRECT_WRITE_ACTION);
		//		fsi.setWritePosition(position + Configuration.getIdBlockSize() - 1, writeInTransaction);
		//		fsi.writeByte((byte) 0, writeInTransaction);
		//
		//		if (Configuration.isDebugEnabled(LOG_ID_DEBUG)) {
		//			DLogger.debug(depthToSpaces() + "After create block, available position is " + fsi.getAvailablePosition());
		//		}
		//
		//		return position;
		//	}
		//
		//	/**
		//	 * Marks a block of type id as full, changes the status and the next block
		//	 * position
		//	 * 
		//	 * @param blockPosition
		//	 * @param nextBlockPosition
		//	 * @param writeInTransaction
		//	 * @return The block position
		//	 * @throws IOException
		//	 */
		//	public long markIdBlockAsFull(long blockPosition, long nextBlockPosition, boolean writeInTransaction) throws IOException {
		//		fsi.setWritePosition(blockPosition + StorageEngineConstant.BLOCK_ID_OFFSET_FOR_BLOCK_STATUS, writeInTransaction);
		//		fsi.writeByte(BlockStatus.BLOCK_FULL, writeInTransaction);
		//		fsi.setWritePosition(blockPosition + StorageEngineConstant.BLOCK_ID_OFFSET_FOR_NEXT_BLOCK, writeInTransaction);
		//		fsi.writeLong(nextBlockPosition, writeInTransaction, "next id block pos", DefaultWriteAction.DIRECT_WRITE_ACTION);
		//		return blockPosition;
		//	}
		//
		//	public long associateIdToObject(byte idType, byte idStatus, long currentBlockIdPosition, OID nextId, long objectPosition,
		//			boolean writeInTransaction) throws IOException {
		//
		//		// Update the max id of the current block
		//		fsi.setWritePosition(currentBlockIdPosition + StorageEngineConstant.BLOCK_ID_OFFSET_FOR_MAX_ID, writeInTransaction);
		//		fsi.writeLong(nextId.getObjectId(), writeInTransaction, "id block max id update", DefaultWriteAction.POINTER_WRITE_ACTION);
		//
		//		long l1 = (nextId.getObjectId() - 1) % Configuration.getNB_IDS_PER_BLOCK();
		//		long l2 = StorageEngineConstant.BLOCK_ID_OFFSET_FOR_START_OF_REPETITION;
		//		long idPosition = currentBlockIdPosition + StorageEngineConstant.BLOCK_ID_OFFSET_FOR_START_OF_REPETITION + (l1)
		//				* Configuration.getID_BLOCK_REPETITION_SIZE();
		//		// go to the next id position
		//		fsi.setWritePosition(idPosition, writeInTransaction);
		//		// id type
		//		fsi.writeByte(idType, writeInTransaction, "id type");
		//		// id
		//		fsi.writeLong(nextId.getObjectId(), writeInTransaction, "oid", DefaultWriteAction.POINTER_WRITE_ACTION);
		//		// id status
		//		fsi.writeByte(idStatus, writeInTransaction, "id status");
		//		// object position
		//		fsi.writeLong(objectPosition, writeInTransaction, "obj pos", DefaultWriteAction.POINTER_WRITE_ACTION);
		//
		//		return idPosition;
		//	}
		//
		//	public void updateObjectPositionForIdWithPosition(long idPosition, long objectPosition, boolean writeInTransaction) throws IOException {
		//		fsi.setWritePosition(idPosition, writeInTransaction);
		//		fsi.writeByte(IDTypes.OBJECT, writeInTransaction, "id type");
		//		fsi.setWritePosition(idPosition + StorageEngineConstant.BLOCK_ID_REPETITION_ID_STATUS, writeInTransaction);
		//		fsi.writeByte(IDStatus.ACTIVE, writeInTransaction);
		//		fsi.writeLong(objectPosition, writeInTransaction, "Updating object position of id", DefaultWriteAction.POINTER_WRITE_ACTION);
		//	}
		//
		//	public void updateClassPositionForIdWithPosition(long idPosition, long objectPosition, boolean writeInTransaction) throws IOException {
		//		fsi.setWritePosition(idPosition, writeInTransaction);
		//		fsi.writeByte(IDTypes.CLASS, writeInTransaction, "id type");
		//		fsi.setWritePosition(idPosition + StorageEngineConstant.BLOCK_ID_REPETITION_ID_STATUS, writeInTransaction);
		//		fsi.writeByte(IDStatus.ACTIVE, writeInTransaction);
		//		fsi.writeLong(objectPosition, writeInTransaction, "Updating class position of id", DefaultWriteAction.POINTER_WRITE_ACTION);
		//	}
		//
		//	public void updateStatusForIdWithPosition(long idPosition, byte newStatus, boolean writeInTransaction) throws IOException {
		//		fsi.setWritePosition(idPosition + StorageEngineConstant.BLOCK_ID_REPETITION_ID_STATUS, writeInTransaction);
		//		fsi.writeByte(newStatus, writeInTransaction, "Updating id status");
		//	}
		//
		//	/**
		//	 * Write the class info header to the database file
		//	 * 
		//	 * @param classInfo
		//	 *            The class info to be written
		//	 * @param position
		//	 *            The position at which it must be written
		//	 * @param writeInTransaction
		//	 *            true if the write must be done in transaction, false to write
		//	 *            directly
		//	 * @throws IOException
		//	 */
		//	public void writeClassInfoHeader(ClassInfo classInfo, long position, boolean writeInTransaction) throws IOException {
		//
		//		OID classId = classInfo.getId();
		//		if (classId == null) {
		//			classId = idManager.getNextClassId(position);
		//			classInfo.setId(classId);
		//		} else {
		//			idManager.updateClassPositionForId(classId, position, true);
		//		}
		//
		//		fsi.setWritePosition(position, writeInTransaction);
		//		if (Configuration.isDebugEnabled(LOG_ID)) {
		//			DLogger.debug(depthToSpaces() + "Writing new Class info header at " + position + " : " + classInfo.toString());
		//		}
		//
		//		// Real value of block size is only known at the end of the writing
		//		fsi.writeInt(0, writeInTransaction, "block size");
		//		fsi.writeByte(BlockTypes.BLOCK_TYPE_CLASS_HEADER, writeInTransaction, "class header block type");
		//
		//		fsi.writeByte(classInfo.getClassCategory(), writeInTransaction, "Class info category");
		//		fsi.writeLong(classId.getObjectId(), writeInTransaction, "class id", DefaultWriteAction.DATA_WRITE_ACTION);
		//
		//		writeOid(classInfo.getPreviousClassOID(), writeInTransaction, "prev class oid", DefaultWriteAction.DATA_WRITE_ACTION);
		//		writeOid(classInfo.getNextClassOID(), writeInTransaction, "next class oid", DefaultWriteAction.DATA_WRITE_ACTION);
		//
		//		fsi
		//				.writeLong(classInfo.getCommitedZoneInfo().getNbObjects(), writeInTransaction, "class nb objects",
		//						DefaultWriteAction.DATA_WRITE_ACTION);
		//
		//		writeOid(classInfo.getCommitedZoneInfo().first, writeInTransaction, "class first obj pos", DefaultWriteAction.DATA_WRITE_ACTION);
		//		writeOid(classInfo.getCommitedZoneInfo().last, writeInTransaction, "class last obj pos", DefaultWriteAction.DATA_WRITE_ACTION);
		//
		//		fsi.writeString(classInfo.getClassName(), writeInTransaction);
		//		fsi.writeString(classInfo.getPackageName(), writeInTransaction);
		//
		//		fsi.writeInt(classInfo.getMaxAttributeId(), writeInTransaction, "Max attribute id");
		//
		//		if (classInfo.getAttributesDefinitionPosition() != -1) {
		//			fsi.writeLong(classInfo.getAttributesDefinitionPosition(), writeInTransaction, "class att def pos",
		//					DefaultWriteAction.DATA_WRITE_ACTION);
		//		} else {
		//			// @todo check this
		//			fsi.writeLong(-1, writeInTransaction, "class att def pos", DefaultWriteAction.DATA_WRITE_ACTION);
		//		}
		//
		//		int blockSize = (int) (fsi.getPosition() - position);
		//		writeBlockSizeAt(position, blockSize, writeInTransaction, classInfo);
		//
		//	}
		//
		//	private void writeOid(OID oid, boolean writeInTransaction, String label, int writeAction) throws IOException {
		//		if (oid == null) {
		//			fsi.writeLong(-1, writeInTransaction, label, writeAction);
		//		} else {
		//			fsi.writeLong(oid.getObjectId(), writeInTransaction, label, writeAction);
		//		}
		//	}
		//
		//	/**
		//	 * Write the class info body to the database file. TODO Check if we really
		//	 * must recall the writeClassInfoHeader
		//	 * 
		//	 * @param classInfo
		//	 * @param position
		//	 *            The position
		//	 * @param writeInTransaction
		//	 * @throws IOException
		//	 */
		//	protected void writeClassInfoBody(ClassInfo classInfo, long position, boolean writeInTransaction) throws IOException {
		//		if (Configuration.isDebugEnabled(LOG_ID)) {
		//			DLogger.debug(depthToSpaces() + "Writing new Class info body at " + position + " : " + classInfo.toString());
		//		}
		//
		//		// updates class info
		//		classInfo.setAttributesDefinitionPosition(position);
		//		// @todo : change this to right only the postion and not the whole
		//		// header
		//		writeClassInfoHeader(classInfo, classInfo.getPosition(), writeInTransaction);
		//
		//		fsi.setWritePosition(position, writeInTransaction);
		//		// block definition
		//		fsi.writeInt(0, writeInTransaction, "block size");
		//		fsi.writeByte(BlockTypes.BLOCK_TYPE_CLASS_BODY, writeInTransaction);
		//
		//		// number of attributes
		//		fsi.writeLong(classInfo.getAttributes().size(), writeInTransaction, "class nb attributes", DefaultWriteAction.DATA_WRITE_ACTION);
		//
		//		ClassAttributeInfo cai = null;
		//		for (int i = 0; i < classInfo.getAttributes().size(); i++) {
		//			cai = (ClassAttributeInfo) classInfo.getAttributes().get(i);
		//			writeClassAttributeInfo(cai, writeInTransaction);
		//		}
		//		int blockSize = (int) (fsi.getPosition() - position);
		//		writeBlockSizeAt(position, blockSize, writeInTransaction, classInfo);
		//	}
		//
		//	public long writeClassInfoIndexes(ClassInfo classInfo) throws IOException {
		//		boolean writeInTransaction = true;
		//		long position = fsi.getAvailablePosition();
		//		fsi.setWritePosition(position, writeInTransaction);
		//		ClassInfoIndex cii = null;
		//		long previousIndexPosition = -1;
		//		long currentIndexPosition = position;
		//		long nextIndexPosition = -1;
		//		long currentPosition = -1;
		//		for (int i = 0; i < classInfo.getNumberOfIndexes(); i++) {
		//			currentIndexPosition = fsi.getPosition();
		//			cii = classInfo.getIndex(i);
		//			fsi.writeInt(0, writeInTransaction, "block size");
		//			fsi.writeByte(BlockTypes.BLOCK_TYPE_INDEX, true, "Index block type");
		//
		//			fsi.writeLong(previousIndexPosition, writeInTransaction, "prev index pos", DefaultWriteAction.POINTER_WRITE_ACTION);
		//			// The next position is only know at the end of the write
		//			fsi.writeLong(-1, writeInTransaction, "next index pos", DefaultWriteAction.POINTER_WRITE_ACTION);
		//
		//			fsi.writeString(cii.getName(), writeInTransaction);
		//			fsi.writeBoolean(cii.isUnique(), writeInTransaction, "index is unique");
		//			fsi.writeByte(cii.getStatus(), writeInTransaction, "index status");
		//			fsi.writeLong(cii.getCreationDate(), writeInTransaction, "creation date", DefaultWriteAction.DATA_WRITE_ACTION);
		//			fsi.writeLong(cii.getLastRebuild(), writeInTransaction, "last rebuild", DefaultWriteAction.DATA_WRITE_ACTION);
		//			fsi.writeInt(cii.getAttributeIds().length, writeInTransaction, "number of fields");
		//			for (int j = 0; j < cii.getAttributeIds().length; j++) {
		//				fsi.writeInt(cii.getAttributeIds()[j], writeInTransaction, "attr id");
		//			}
		//			currentPosition = fsi.getPosition();
		//			// Write the block size
		//			int blockSize = (int) (fsi.getPosition() - currentIndexPosition);
		//			writeBlockSizeAt(currentIndexPosition, blockSize, writeInTransaction, classInfo);
		//
		//			// Write the next index position
		//			if (i + 1 < classInfo.getNumberOfIndexes()) {
		//				nextIndexPosition = currentPosition;
		//			} else {
		//				nextIndexPosition = -1;
		//			}
		//			// reset cursor to write the next position
		//			fsi.setWritePosition(currentIndexPosition + ODBType.INTEGER.getSize() + ODBType.BYTE.getSize() + ODBType.LONG.getSize(),
		//					writeInTransaction);
		//			fsi.writeLong(nextIndexPosition, writeInTransaction, "next index pos", DefaultWriteAction.POINTER_WRITE_ACTION);
		//			previousIndexPosition = currentIndexPosition;
		//
		//			// reset the write cursor
		//			fsi.setWritePosition(currentPosition, writeInTransaction);
		//
		//		}
		//
		//		return position;
		//
		//	}
		//
		//	public void updateClassInfo(ClassInfo classInfo, boolean writeInTransaction) throws IOException {
		//		// To force the rewrite of class info body
		//		classInfo.setAttributesDefinitionPosition(-1);
		//		long newCiPosition = fsi.getAvailablePosition();
		//		classInfo.setPosition(newCiPosition);
		//		writeClassInfoHeader(classInfo, newCiPosition, writeInTransaction);
		//		writeClassInfoBody(classInfo, fsi.getAvailablePosition(), writeInTransaction);
		//	}
		//
		//	/**
		//	 * Resets the position of the first class of the metamodel. It Happens when
		//	 * database is being refactored
		//	 * 
		//	 * @param classInfoPosition
		//	 * @throws IOException
		//	 */
		//	public void writeFirstClassInfoOID(OID classInfoID, boolean inTransaction) throws IOException {
		//		long positionToWrite = StorageEngineConstant.DATABASE_HEADER_FIRST_CLASS_OID;
		//		fsi.setWritePosition(positionToWrite, inTransaction);
		//		writeOid(classInfoID, inTransaction, "first class info oid", DefaultWriteAction.DATA_WRITE_ACTION);
		//		if (Configuration.isDebugEnabled(LOG_ID)) {
		//			DLogger.debug(depthToSpaces() + "Updating first class info oid at " + positionToWrite + " with oid " + classInfoID);
		//		}
		//	}
		//
		//	private void updateNextClassInfoPositionOfClassInfo(long classInfoPosition, long newCiPosition) throws IOException {
		//		fsi.setWritePosition(classInfoPosition + StorageEngineConstant.CLASS_OFFSET_NEXT_CLASS_POSITION, true);
		//		fsi.writeLong(newCiPosition, true, "new next ci position", DefaultWriteAction.DATA_WRITE_ACTION);
		//		if (Configuration.isDebugEnabled(LOG_ID)) {
		//			DLogger
		//					.debug(depthToSpaces() + "Updating next class info of class info at " + classInfoPosition + " with "
		//							+ classInfoPosition);
		//		}
		//	}
		//
		//	private void updatePreviousClassInfoPositionOfClassInfo(long classInfoPosition, long newCiPosition) throws IOException {
		//		fsi.setWritePosition(classInfoPosition + StorageEngineConstant.CLASS_OFFSET_PREVIOUS_CLASS_POSITION, true);
		//		fsi.writeLong(newCiPosition, true, "new prev ci position", DefaultWriteAction.DATA_WRITE_ACTION);
		//		if (Configuration.isDebugEnabled(LOG_ID)) {
		//			DLogger
		//					.debug(depthToSpaces() + "Updating prev class info of class info at " + classInfoPosition + " with "
		//							+ classInfoPosition);
		//		}
		//	}
		//
		//	/**
		//	 * Writes a class attribute info, an attribute of a class
		//	 * 
		//	 * @param cai
		//	 * @param writeInTransaction
		//	 * @throws IOException
		//	 */
		//	private void writeClassAttributeInfo(ClassAttributeInfo cai, boolean writeInTransaction) throws IOException {
		//		fsi.writeInt(cai.getId(), writeInTransaction, "attribute id");
		//		fsi.writeBoolean(cai.isNative(), writeInTransaction);
		//		if (cai.isNative()) {
		//			fsi.writeInt(cai.getAttributeType().getId(), writeInTransaction, "att odb type id");
		//			if (cai.getAttributeType().isArray()) {
		//				fsi.writeInt(cai.getAttributeType().getSubType().getId(), writeInTransaction, "att array sub type");
		//				// when the attribute is not native, then write its class info
		//				// position
		//				if (cai.getAttributeType().getSubType().isNonNative()) {
		//					fsi.writeLong(storageEngine.getSession(true).getMetaModel().getClassInfo(cai.getAttributeType().getSubType().getName(), true).getId()
		//							.getObjectId(), writeInTransaction, "class info id of array subtype", DefaultWriteAction.DATA_WRITE_ACTION);
		//				}
		//			}
		//		} else {
		//			fsi.writeLong(storageEngine.getSession(true).getMetaModel().getClassInfo(cai.getFullClassname(), true).getId().getObjectId(),
		//					writeInTransaction, "class info id", DefaultWriteAction.DATA_WRITE_ACTION);
		//		}
		//		fsi.writeString(cai.getName(), writeInTransaction);
		//		fsi.writeBoolean(cai.isIndex(), writeInTransaction);
		//	}
		//
		//	/*
		//	 * protected long writeObjectInfo(long oid, AbstractObjectInfo objectInfo,
		//	 * boolean updatePointers) throws Exception { return writeObjectInfo(oid,
		//	 * objectInfo, -1, updatePointers); }
		//	 */
		//
		//	/**
		//	 * Actually write the object data to the database file
		//	 * 
		//	 * @param oid
		//	 *            The object id, can be -1 (not set)
		//	 * @param aoi
		//	 *            The object meta infor The object info to be written
		//	 * @param position
		//	 *            if -1, it is a new instance, if not, it is an update
		//	 * @param updatePointers
		//	 * @return The object posiiton or id(if <0)
		//	 * @throws Exception
		//	 * @throws IOException
		//	 * 
		//	 * public OID writeObjectInfo(OID oid, AbstractObjectInfo aoi, long
		//	 * position, boolean updatePointers) throws Exception { currentDepth++;
		//	 * 
		//	 * try {
		//	 * 
		//	 * if (aoi.isNative()) { return writeNativeObjectInfo((NativeObjectInfo)
		//	 * aoi, position, updatePointers, false); }
		//	 * 
		//	 * return writeNonNativeObjectInfo(oid, aoi, position, updatePointers,
		//	 * false); } finally { currentDepth--; } }
		//	 */
		//
		//	private long writeNativeObjectInfo(NativeObjectInfo noi, long position, boolean updatePointers, boolean writeInTransaction)
		//			throws Exception {
		//
		//		if (Configuration.isDebugEnabled(LOG_ID_DEBUG)) {
		//			DLogger.debug(depthToSpaces() + "Writing native object at " + position + " : Type=" + ODBType.getNameFromId(noi.getOdbTypeId())
		//					+ " | Value=" + noi.toString());
		//		}
		//
		//		if (noi.isAtomicNativeObject()) {
		//			return writeAtomicNativeObject((AtomicNativeObjectInfo) noi, writeInTransaction);
		//		}
		//
		//		if (noi.isNull()) {
		//			writeNullNativeObjectHeader(noi.getOdbTypeId(), writeInTransaction);
		//			return position;
		//		}
		//
		//		if (noi.isCollectionObject()) {
		//			return writeCollection((CollectionObjectInfo) noi, writeInTransaction);
		//		}
		//		if (noi.isMapObject()) {
		//			return writeMap((MapObjectInfo) noi, writeInTransaction);
		//		}
		//		if (noi.isArrayObject()) {
		//			return writeArray((ArrayObjectInfo) noi, writeInTransaction);
		//		}
		//
		//		throw new ODBRuntimeException(Error.NATIVE_TYPE_NOT_SUPPORTED.addParameter(noi.getOdbTypeId()));
		//	}
		//
		//	/**
		//	 * Write an object representation to database file
		//	 * 
		//	 * @param existingOid
		//	 *            The oid of the object, can be null
		//	 * @param aoi
		//	 *            The Object meta representation
		//	 * @param position
		//	 *            The position where the object must be written, can be -1
		//	 * @param updatePointers
		//	 *            To indicate if pointers (prev,next) must be updates
		//	 * @param writeDataInTransaction
		//	 *            To indicate if the write must be done in or out of transaction
		//	 * @return The oid of the object
		//	 * @throws Exception
		//	 */
		//	public OID writeNonNativeObjectInfo(OID existingOid, NonNativeObjectInfo objectInfo, long position, boolean writeDataInTransaction,
		//			boolean isNewObject) throws Exception {
		//
		//		ISession lsession = getSession();
		//		ICache cache = lsession.getCache();
		//
		//		// Checks if object is null,for null objects,there is nothing to do
		//		if (objectInfo.isNull()) {
		//			return StorageEngineConstant.NULL_OBJECT_ID;
		//		}
		//
		//		// First store all non native attributes before: Write all dependencies
		//		// first in
		//		// order to have all OIDs ok to write this object in a single block and
		//		// to be able to compute
		//		// The size of the block
		//		List allAttributesNoi = objectInfo.getAllNonNativeAttributes();
		//		Iterator iterator = allAttributesNoi.iterator();
		//		while (iterator.hasNext()) {
		//			Object o = iterator.next();
		//			NonNativeObjectInfo nnoi = (NonNativeObjectInfo) o;
		//			OID nnoiOid = nnoi.getOid();
		//			OID oid = internalStoreObject(nnoiOid, nnoi);
		//		}
		//
		//		MetaModel metaModel = lsession.getMetaModel();
		//
		//		// first checks if the class of this object already exist in the
		//		// metamodel
		//		if (!metaModel.existClass(objectInfo.getClassInfo().getFullClassName())) {
		//			storageEngine.getObjectWriter().addClass(objectInfo.getClassInfo(), true);
		//		}
		//
		//		// if position is -1, gets the position where to write the object
		//		if (position == -1) {
		//			// Write at the end of the file
		//			position = fsi.getAvailablePosition();
		//			// Updates the meta object position
		//			objectInfo.setPosition(position);
		//		}
		//
		//		// Gets the object id
		//		OID oid = existingOid;
		//
		//		if (oid == null) {
		//			// If, to get the next id, a new id block must be created, then
		//			// there is an extra work
		//			// to update the current object position
		//			if (idManager.mustShift()) {
		//				oid = idManager.getNextObjectId(position);
		//				// The id manager wrote in the file so the position for the
		//				// object must be re-computed
		//				position = fsi.getAvailablePosition();
		//				// The oid must be associated to this new position - id
		//				// operations are always out of transaction
		//				// in this case, the update is done out of the transaction as a
		//				// rollback won t need to
		//				// undo this. We are just creating the id
		//				// => third parameter(write in transaction) = false
		//				idManager.updateObjectPositionForOid(oid, position, false);
		//
		//			} else {
		//				oid = idManager.getNextObjectId(position);
		//			}
		//		} else {
		//			// If an oid was passed, it is because object already exist and
		//			// is being updated. So we
		//			// must update the object position
		//			// Here the update of the position of the id must be done in
		//			// transaction as the object
		//			// position of the id is being updated, and a rollback should undo
		//			// this
		//			// => third parameter(write in transaction) = true
		//			idManager.updateObjectPositionForOid(oid, position, true);
		//			// Keep the relation of id and position in the cache until the
		//			// commit
		//			cache.savePositionOfObjectWithOid(oid, position);
		//		}
		//
		//		// Sets the oid of the object in the inserting cache
		//		cache.updateIdOfInsertingObject(objectInfo.getObject(), oid);
		//		// Only add the oid to unconnected zone if it is a new object
		//		if (isNewObject) {
		//			cache.addOIDToUnconnectedZone(oid);
		//		}
		//
		//		objectInfo.setOid(oid);
		//
		//		if (Configuration.isDebugEnabled(LOG_ID)) {
		//			DLogger.debug(depthToSpaces() + "Start Writing non native object of type " + objectInfo.getClassInfo().getClassName() + " at "
		//					+ position + " , oid = " + oid + " : " + objectInfo.toString());
		//		}
		//
		//		if (objectInfo.getClassInfo() == null || objectInfo.getClassInfo().getId() == null) {
		//
		//			if (objectInfo.getClassInfo() != null) {
		//				ClassInfo clinfo = storageEngine.getSession(true).getMetaModel().getClassInfo(objectInfo.getClassInfo().getFullClassName(), true);
		//				objectInfo.setClassInfo(clinfo);
		//			} else {
		//				throw new ODBRuntimeException(Error.UNDEFINED_CLASS_INFO.addParameter(objectInfo.toString()));
		//			}
		//		}
		//
		//		// updates the meta model - If class already exist, it returns the
		//		// metamodel class, which contains
		//		// a bit more informations
		//		ClassInfo classInfo = storageEngine.getObjectWriter().addClass(objectInfo.getClassInfo(), true);
		//		objectInfo.setClassInfo(classInfo);
		//
		//		// 
		//
		//		if (isNewObject) {
		//			manageNewObjectPointers(objectInfo, classInfo, position, metaModel);
		//		}
		//		/*
		//		 * else{ throw new
		//		 * ODBRuntimeException(Error.UNEXPECTED_SITUATION.addParameter("WritingNonNativeObject
		//		 * that is not new and without updating pointers")); }
		//		 */
		//
		//		if (Configuration.saveHistory()) {
		//			classInfo.addHistory(new InsertHistoryInfo("insert", oid, position, objectInfo.getPreviousObjectOID(), objectInfo
		//					.getNextObjectOID()));
		//		}
		//
		//		fsi.setWritePosition(position, writeDataInTransaction);
		//		objectInfo.setPosition(position);
		//
		//		// Block size
		//		fsi.writeInt(0, writeDataInTransaction, "block size");
		//		// Block type
		//		fsi.writeByte(BlockTypes.BLOCK_TYPE_NON_NATIVE_OBJECT, writeDataInTransaction, "object block type");
		//		// The object id
		//		fsi.writeLong(oid.getObjectId(), writeDataInTransaction, "oid", DefaultWriteAction.DATA_WRITE_ACTION);
		//		// Class info id
		//		fsi.writeLong(classInfo.getId().getObjectId(), writeDataInTransaction, "class info id", DefaultWriteAction.DATA_WRITE_ACTION);
		//
		//		// previous instance
		//		writeOid(objectInfo.getPreviousObjectOID(), writeDataInTransaction, "prev instance", DefaultWriteAction.DATA_WRITE_ACTION);
		//		// next instance
		//		writeOid(objectInfo.getNextObjectOID(), writeDataInTransaction, "next instance", DefaultWriteAction.DATA_WRITE_ACTION);
		//		// creation date, for update operation must be the original one
		//		fsi.writeLong(OdbTime.getCurrentTimeInMs(), writeDataInTransaction, "creation date", DefaultWriteAction.DATA_WRITE_ACTION);
		//		fsi.writeLong(OdbTime.getCurrentTimeInMs(), writeDataInTransaction, "update date", DefaultWriteAction.DATA_WRITE_ACTION);
		//		// TODO check next version number
		//		fsi.writeInt(objectInfo.getHeader().getObjectVersion() + 1, writeDataInTransaction, "object version number");
		//
		//		// not used yet. But it will point to an internal object of type
		//		// ObjectReference that will have details on the references:
		//		// All the objects that point to it: to enable object integrity
		//		fsi.writeLong(-1, writeDataInTransaction, "object reference pointer", DefaultWriteAction.DATA_WRITE_ACTION);
		//
		//		// True if this object have been synchronized with main database, else
		//		// false
		//		fsi.writeBoolean(false, writeDataInTransaction, "is syncronized with external db");
		//
		//		int nbAttributes = objectInfo.getClassInfo().getAttributes().size();
		//
		//		// now write the number of attributes and the position of all
		//		// attributes, we do not know them yet, so write 00 but at the end
		//		// of the write operation
		//		// These positions will be updated
		//		// The positions that is going to be written are 'int' representing
		//		// the offset position of the attribute
		//		// first write the number of attributes
		//		fsi.writeInt(nbAttributes, writeDataInTransaction, "nb attr");
		//
		//		// Store the position
		//		long attributePositionStart = fsi.getPosition();
		//
		//		// TODO Could remove this, and pull to the right position
		//		for (int i = 0; i < nbAttributes; i++) {
		//			fsi.writeInt(0, writeDataInTransaction, "attr id -1");
		//			fsi.writeLong(0, writeDataInTransaction, "att pos", DefaultWriteAction.DATA_WRITE_ACTION);
		//		}
		//
		//		long[] attributesIdentification = new long[nbAttributes];
		//		int[] attributeIds = new int[nbAttributes];
		//
		//		// Puts the object info in the cache
		//		// storageEngine.getSession().getCache().addObject(position,
		//		// aoi.getObject(), objectInfo.getHeader());
		//
		//		ClassAttributeInfo cai = null;
		//		AbstractObjectInfo aoi2 = null;
		//		long nativeAttributePosition = -1;
		//		OID nonNativeAttributeOid = null;
		//		long maxWritePosition = fsi.getPosition();
		//
		//		// Loop on all attributes
		//		for (int i = 0; i < nbAttributes; i++) {
		//			// Gets the attribute meta description
		//			cai = classInfo.getAttributeInfo(i);
		//			// Gets the id of the attribute
		//			attributeIds[i] = cai.getId();
		//			// Gets the attribute data
		//			aoi2 = (AbstractObjectInfo) objectInfo.getAttributeValueFromId(cai.getId());
		//
		//			if (Configuration.isDebugEnabled(LOG_ID)) {
		//				DLogger.debug(depthToSpaces() + "  Writing attribute " + cai.getName() + " of type " + cai.getClassName());
		//			}
		//
		//			if (aoi2 == null) {
		//				// This only happens in 1 case : when a class has a field with
		//				// the same name of one of is superclass. In this, the deeper
		//				// attribute is null
		//				if (cai.isNative()) {
		//					aoi2 = new NullNativeObjectInfo(cai.getAttributeType().getId());
		//				} else {
		//					aoi2 = new NonNativeNullObjectInfo(cai.getClassInfo());
		//				}
		//			}
		//
		//			if (aoi2.isNative()) {
		//				nativeAttributePosition = internalStoreObject((NativeObjectInfo) aoi2);
		//				// For native objects , odb stores their position
		//				attributesIdentification[i] = nativeAttributePosition;
		//
		//			} else {
		//				if (aoi2.isObjectReference()) {
		//					ObjectReference or = (ObjectReference) aoi2;
		//					nonNativeAttributeOid = or.getOid();
		//				} else {
		//					// nonNativeAttributeOid = internalStoreObject(null,
		//					// (NonNativeObjectInfo) aoi2);
		//					NonNativeObjectInfo nnoi = (NonNativeObjectInfo) aoi2;
		//					nonNativeAttributeOid = nnoi.getOid();
		//				}
		//				// For non native objects , odb stores its oid as a negative
		//				// number!!
		//				if (nonNativeAttributeOid != null) {
		//					attributesIdentification[i] = -nonNativeAttributeOid.getObjectId();
		//				} else {
		//					attributesIdentification[i] = StorageEngineConstant.NULL_OBJECT_ID_ID;
		//				}
		//			}
		//
		//			long p = fsi.getPosition();
		//			if (p > maxWritePosition) {
		//				maxWritePosition = p;
		//			}
		//		}
		//
		//		// Updates attributes identification in the object info header
		//		objectInfo.getHeader().setAttributesIdentification(attributesIdentification);
		//		objectInfo.getHeader().setAttributesIds(attributeIds);
		//
		//		long positionAfterWrite = maxWritePosition;
		//
		//		// Now writes back the attribute positions
		//		fsi.setWritePosition(attributePositionStart, writeDataInTransaction);
		//
		//		for (int i = 0; i < attributesIdentification.length; i++) {
		//			fsi.writeInt(attributeIds[i], writeDataInTransaction, "attr id");
		//			fsi.writeLong(attributesIdentification[i], writeDataInTransaction, "att real pos", DefaultWriteAction.DATA_WRITE_ACTION);
		//
		//			// if (classInfo.getAttributeInfo(i).isNonNative() &&
		//			// attributesIdentification[i] > 0) {
		//			if (objectInfo.getAttributeValueFromId(attributeIds[i]).isNonNativeObject() && attributesIdentification[i] > 0) {
		//				throw new ODBRuntimeException(Error.NON_NATIVE_ATTRIBUTE_STORED_BY_POSITION_INSTEAD_OF_OID.addParameter(
		//						classInfo.getAttributeInfo(i).getName()).addParameter(classInfo.getFullClassName()).addParameter(
		//						attributesIdentification[i]));
		//			}
		//		}
		//		fsi.setWritePosition(positionAfterWrite, writeDataInTransaction);
		//
		//		int blockSize = (int) (positionAfterWrite - position);
		//
		//		try {
		//			writeBlockSizeAt(position, blockSize, writeDataInTransaction, objectInfo);
		//		} catch (ODBRuntimeException e) {
		//			DLogger.debug("Error while writing block size. pos after write " + positionAfterWrite + " / start pos = " + position);
		//			// throw new ODBRuntimeException(storageEngine,"Error while writing
		//			// block size. pos after write " + positionAfterWrite + " / start
		//			// pos = " + position,e);
		//			throw e;
		//		}
		//
		//		if (Configuration.isDebugEnabled(LOG_ID)) {
		//			DLogger.debug(depthToSpaces() + "  Attributes positions of object with oid " + oid + " are "
		//					+ DisplayUtility.longArrayToString(attributesIdentification));
		//			DLogger.debug(depthToSpaces() + "End Writing non native object at " + position + " with oid " + oid + " - prev oid="
		//					+ objectInfo.getPreviousObjectOID() + " / next oid=" + objectInfo.getNextObjectOID());
		//			if (Configuration.isDebugEnabled(LOG_ID_DEBUG)) {
		//				DLogger.debug(" - current buffer : " + fsi.getIo().toString());
		//			}
		//		}
		//
		//		// Only insert in index for new objects
		//		if (isNewObject) {
		//			// insert object id in indexes, if exist
		//			manageIndexesForInsert(oid, objectInfo);
		//		}
		//
		//		return oid;
		//	}
		//
		//	/**
		//	 * Write an object representation to database file
		//	 * 
		//	 * @param existingOid
		//	 *            The oid of the object, can be null
		//	 * @param aoi
		//	 *            The Object meta representation
		//	 * @param position
		//	 *            The position where the object must be written, can be -1
		//	 * @param updatePointers
		//	 *            To indicate if pointers (prev,next) must be updates
		//	 * @param writeDataInTransaction
		//	 *            To indicate if the write must be done in or out of transaction
		//	 * @return The oid of the object
		//	 * @throws Exception
		//	 */
		//	public OID writeNonNativeObjectInfoOld(OID existingOid, NonNativeObjectInfo objectInfo, long position, boolean writeDataInTransaction,
		//			boolean isNewObject) throws Exception {
		//
		//		ISession lsession = getSession();
		//		ICache cache = lsession.getCache();
		//
		//		// Checks if object is null,for null objects,there is nothing to do
		//		if (objectInfo.isNull()) {
		//			return StorageEngineConstant.NULL_OBJECT_ID;
		//		}
		//
		//		MetaModel metaModel = lsession.getMetaModel();
		//
		//		// first checks if the class of this object already exist in the
		//		// metamodel
		//		if (!metaModel.existClass(objectInfo.getClassInfo().getFullClassName())) {
		//			storageEngine.getObjectWriter().addClass(objectInfo.getClassInfo(), true);
		//		}
		//
		//		// if position is -1, gets the position where to write the object
		//		if (position == -1) {
		//			// Write at the end of the file
		//			position = fsi.getAvailablePosition();
		//			// Updates the meta object position
		//			objectInfo.setPosition(position);
		//		}
		//
		//		// Gets the object id
		//		OID oid = existingOid;
		//
		//		if (oid == null) {
		//			// If, to get the next id, a new id block must be created, then
		//			// there is an extra work
		//			// to update the current object position
		//			if (idManager.mustShift()) {
		//				oid = idManager.getNextObjectId(position);
		//				// The id manager wrote in the file so the position for the
		//				// object must be re-computed
		//				position = fsi.getAvailablePosition();
		//				// The oid must be associated to this new position - id
		//				// operations are always out of transaction
		//				// in this case, the update is done out of the transaction as a
		//				// rollback won t need to
		//				// undo this. We are just creating the id
		//				// => third parameter(write in transaction) = false
		//				idManager.updateObjectPositionForOid(oid, position, false);
		//
		//			} else {
		//				oid = idManager.getNextObjectId(position);
		//			}
		//		} else {
		//			// If an oid was passed, it is because object already exist and
		//			// is being updated. So we
		//			// must update the object position
		//			// Here the update of the position of the id must be done in
		//			// transaction as the object
		//			// position of the id is being updated, and a rollback should undo
		//			// this
		//			// => third parameter(write in transaction) = true
		//			idManager.updateObjectPositionForOid(oid, position, true);
		//			// Keep the relation of id and position in the cache until the
		//			// commit
		//			cache.savePositionOfObjectWithOid(oid, position);
		//		}
		//
		//		// Sets the oid of the object in the inserting cache
		//		cache.updateIdOfInsertingObject(objectInfo.getObject(), oid);
		//		// Only add the oid to unconnected zone if it is a new object
		//		if (isNewObject) {
		//			cache.addOIDToUnconnectedZone(oid);
		//		}
		//
		//		objectInfo.setOid(oid);
		//		if (Configuration.isDebugEnabled(LOG_ID)) {
		//			DLogger.debug(depthToSpaces() + "Start Writing non native object of type " + objectInfo.getClassInfo().getClassName() + " at "
		//					+ position + " , oid = " + oid + " : " + objectInfo.toString());
		//		}
		//
		//		if (objectInfo.getClassInfo() == null || objectInfo.getClassInfo().getId() == null) {
		//
		//			if (objectInfo.getClassInfo() != null) {
		//				ClassInfo clinfo = storageEngine.getSession(true).getMetaModel().getClassInfo(objectInfo.getClassInfo().getFullClassName(), true);
		//				objectInfo.setClassInfo(clinfo);
		//			} else {
		//				throw new ODBRuntimeException(Error.UNDEFINED_CLASS_INFO.addParameter(objectInfo.toString()));
		//			}
		//		}
		//
		//		// updates the meta model - If class already exist, it returns the
		//		// metamodel class, which contains
		//		// a bit more informations
		//		ClassInfo classInfo = storageEngine.getObjectWriter().addClass(objectInfo.getClassInfo(), true);
		//		objectInfo.setClassInfo(classInfo);
		//
		//		// 
		//
		//		if (isNewObject) {
		//			manageNewObjectPointers(objectInfo, classInfo, position, metaModel);
		//		}
		//		/*
		//		 * else{ throw new
		//		 * ODBRuntimeException(Error.UNEXPECTED_SITUATION.addParameter("WritingNonNativeObject
		//		 * that is not new and without updating pointers")); }
		//		 */
		//
		//		if (Configuration.saveHistory()) {
		//			classInfo.addHistory(new InsertHistoryInfo("insert", oid, position, objectInfo.getPreviousObjectOID(), objectInfo
		//					.getNextObjectOID()));
		//		}
		//
		//		fsi.setWritePosition(position, writeDataInTransaction);
		//		objectInfo.setPosition(position);
		//
		//		// Block size
		//		fsi.writeInt(0, writeDataInTransaction, "block size");
		//		// Block type
		//		fsi.writeByte(BlockTypes.BLOCK_TYPE_NON_NATIVE_OBJECT, writeDataInTransaction, "object block type");
		//		// The object id
		//		fsi.writeLong(oid.getObjectId(), writeDataInTransaction, "oid", DefaultWriteAction.DATA_WRITE_ACTION);
		//		// Class info id
		//		fsi.writeLong(classInfo.getId().getObjectId(), writeDataInTransaction, "class info id", DefaultWriteAction.DATA_WRITE_ACTION);
		//
		//		// previous instance
		//		writeOid(objectInfo.getPreviousObjectOID(), writeDataInTransaction, "prev instance", DefaultWriteAction.DATA_WRITE_ACTION);
		//		// next instance
		//		writeOid(objectInfo.getNextObjectOID(), writeDataInTransaction, "next instance", DefaultWriteAction.DATA_WRITE_ACTION);
		//		// creation date, for update operation must be the original one
		//		fsi.writeLong(OdbTime.getCurrentTimeInMs(), writeDataInTransaction, "creation date", DefaultWriteAction.DATA_WRITE_ACTION);
		//		fsi.writeLong(OdbTime.getCurrentTimeInMs(), writeDataInTransaction, "update date", DefaultWriteAction.DATA_WRITE_ACTION);
		//		// TODO check next version number
		//		fsi.writeInt(objectInfo.getHeader().getObjectVersion() + 1, writeDataInTransaction, "object version number");
		//
		//		// not used yet. But it will point to an internal object of type
		//		// ObjectReference that will have details on the references:
		//		// All the objects that point to it: to enable object integrity
		//		fsi.writeLong(-1, writeDataInTransaction, "object reference pointer", DefaultWriteAction.DATA_WRITE_ACTION);
		//
		//		// True if this object have been synchronized with main database, else
		//		// false
		//		fsi.writeBoolean(false, writeDataInTransaction, "is syncronized with external db");
		//
		//		int nbAttributes = objectInfo.getClassInfo().getAttributes().size();
		//
		//		// now write the number of attributes and the position of all
		//		// attributes, we do not know them yet, so write 00 but at the end
		//		// of the write operation
		//		// These positions will be updated
		//		// The positions that is going to be written are 'int' representing
		//		// the offset position of the attribute
		//		// first write the number of attributes
		//		fsi.writeInt(nbAttributes, writeDataInTransaction, "nb attr");
		//
		//		// Store the position
		//		long attributePositionStart = fsi.getPosition();
		//
		//		// TODO Could remove this, and pull to the right position
		//		for (int i = 0; i < nbAttributes; i++) {
		//			fsi.writeInt(0, writeDataInTransaction, "attr id -1");
		//			fsi.writeLong(0, writeDataInTransaction, "att pos", DefaultWriteAction.DATA_WRITE_ACTION);
		//		}
		//
		//		long[] attributesIdentification = new long[nbAttributes];
		//		int[] attributeIds = new int[nbAttributes];
		//
		//		// Puts the object info in the cache
		//		// storageEngine.getSession().getCache().addObject(position,
		//		// aoi.getObject(), objectInfo.getHeader());
		//
		//		ClassAttributeInfo cai = null;
		//		AbstractObjectInfo aoi2 = null;
		//		long nativeAttributePosition = -1;
		//		OID nonNativeAttributeOid = null;
		//		long maxWritePosition = fsi.getPosition();
		//
		//		// Loop on all attributes
		//		for (int i = 0; i < nbAttributes; i++) {
		//			// Gets the attribute meta description
		//			cai = classInfo.getAttributeInfo(i);
		//			// Gets the id of the attribute
		//			attributeIds[i] = cai.getId();
		//			// Gets the attribute data
		//			aoi2 = (AbstractObjectInfo) objectInfo.getAttributeValueFromId(cai.getId());
		//
		//			if (aoi2 == null) {
		//				// This only happens in 1 case : when a class has a field with
		//				// the same name of one of is superclass. In this, the deeper
		//				// attribute is null
		//				if (cai.isNative()) {
		//					aoi2 = new NullNativeObjectInfo(cai.getAttributeType().getId());
		//				} else {
		//					aoi2 = new NonNativeNullObjectInfo(cai.getClassInfo());
		//				}
		//			}
		//
		//			if (aoi2.isNative()) {
		//				nativeAttributePosition = internalStoreObject((NativeObjectInfo) aoi2);
		//				// For native objects , odb stores their position
		//				attributesIdentification[i] = nativeAttributePosition;
		//
		//			} else {
		//				if (aoi2.isObjectReference()) {
		//					ObjectReference or = (ObjectReference) aoi2;
		//					nonNativeAttributeOid = or.getOid();
		//				} else {
		//					nonNativeAttributeOid = internalStoreObject(null, (NonNativeObjectInfo) aoi2);
		//				}
		//				// For non native objects , odb stores its oid as a negative
		//				// number!!
		//				if (nonNativeAttributeOid != null) {
		//					attributesIdentification[i] = -nonNativeAttributeOid.getObjectId();
		//				} else {
		//					attributesIdentification[i] = StorageEngineConstant.NULL_OBJECT_ID_ID;
		//				}
		//			}
		//
		//			long p = fsi.getPosition();
		//			if (p > maxWritePosition) {
		//				maxWritePosition = p;
		//			}
		//		}
		//
		//		// Updates attributes identification in the object info header
		//		objectInfo.getHeader().setAttributesIdentification(attributesIdentification);
		//		objectInfo.getHeader().setAttributesIds(attributeIds);
		//
		//		long positionAfterWrite = maxWritePosition;
		//
		//		// Now writes back the attribute positions
		//		fsi.setWritePosition(attributePositionStart, writeDataInTransaction);
		//
		//		for (int i = 0; i < attributesIdentification.length; i++) {
		//			fsi.writeInt(attributeIds[i], writeDataInTransaction, "attr id");
		//			fsi.writeLong(attributesIdentification[i], writeDataInTransaction, "att real pos", DefaultWriteAction.DATA_WRITE_ACTION);
		//
		//			// if (classInfo.getAttributeInfo(i).isNonNative() &&
		//			// attributesIdentification[i] > 0) {
		//			if (objectInfo.getAttributeValueFromId(attributeIds[i]).isNonNativeObject() && attributesIdentification[i] > 0) {
		//				throw new ODBRuntimeException(Error.NON_NATIVE_ATTRIBUTE_STORED_BY_POSITION_INSTEAD_OF_OID.addParameter(
		//						classInfo.getAttributeInfo(i).getName()).addParameter(classInfo.getFullClassName()).addParameter(
		//						attributesIdentification[i]));
		//			}
		//		}
		//		fsi.setWritePosition(positionAfterWrite, writeDataInTransaction);
		//
		//		int blockSize = (int) (positionAfterWrite - position);
		//
		//		try {
		//			writeBlockSizeAt(position, blockSize, writeDataInTransaction, objectInfo);
		//		} catch (ODBRuntimeException e) {
		//			DLogger.debug("Error while writing block size. pos after write " + positionAfterWrite + " / start pos = " + position);
		//			// throw new ODBRuntimeException(storageEngine,"Error while writing
		//			// block size. pos after write " + positionAfterWrite + " / start
		//			// pos = " + position,e);
		//			throw e;
		//		}
		//
		//		if (Configuration.isDebugEnabled(LOG_ID)) {
		//			DLogger.debug(depthToSpaces() + "  Attributes positions of object with oid " + oid + " are "
		//					+ DisplayUtility.longArrayToString(attributesIdentification));
		//			DLogger.debug(depthToSpaces() + "End Writing non native object at " + position + " with oid " + oid + " - prev oid="
		//					+ objectInfo.getPreviousObjectOID() + " / next oid=" + objectInfo.getNextObjectOID());
		//			if (Configuration.isDebugEnabled(LOG_ID_DEBUG)) {
		//				DLogger.debug(" - current buffer : " + fsi.getIo().toString());
		//			}
		//		}
		//
		//		// Only insert in index for new objects
		//		if (isNewObject) {
		//			// insert object id in indexes, if exist
		//			manageIndexesForInsert(oid, objectInfo);
		//		}
		//
		//		return oid;
		//	}
		//
		//	/**
		//	 * Updates pointers of objects, Only changes uncommited info pointers
		//	 * 
		//	 * @param objectInfo
		//	 *            The meta representation of the object being inserted
		//	 * @param classInfo
		//	 *            The class of the object being inserted
		//	 * @param position
		//	 *            The position where the object is being inserted
		//	 * @throws IOException
		//	 */
		//	private void manageNewObjectPointers(NonNativeObjectInfo objectInfo, ClassInfo classInfo, long position, MetaModel metaModel)
		//			throws IOException {
		//		ICache cache = storageEngine.getSession(true).getCache();
		//		boolean isFirstUncommitedObject = !classInfo.getUncommittedZoneInfo().hasObjects();
		//		// if it is the first uncommitted object
		//		if (isFirstUncommitedObject) {
		//			classInfo.getUncommittedZoneInfo().first = objectInfo.getOid();
		//			OID lastCommittedObjectOid = classInfo.getCommitedZoneInfo().last;
		//			if (lastCommittedObjectOid != null) {
		//				// Also updates the last committed object next object oid in
		//				// memory to connect the committed
		//				// zone with unconnected for THIS transaction (only in memory)
		//				ObjectInfoHeader oih = cache.getObjectInfoHeaderFromOid(lastCommittedObjectOid, true);
		//				oih.setNextObjectOID(objectInfo.getOid());
		//
		//				// And sets the previous oid of the current object with the last
		//				// committed oid
		//				objectInfo.setPreviousInstanceOID(lastCommittedObjectOid);
		//			}
		//		} else {
		//			// Gets the last object, updates its (next object)
		//			// pointer to the new object and updates the class info 'last
		//			// uncommitted object
		//			// oid' field
		//			ObjectInfoHeader oip = classInfo.getLastObjectInfoHeader();
		//
		//			if (oip == null) {
		//				throw new ODBRuntimeException(Error.INTERNAL_ERROR.addParameter("last OIP is null in manageNewObjectPointers oid="
		//						+ objectInfo.getOid()));
		//			}
		//			if (oip.getNextObjectOID() != objectInfo.getOid()) {
		//				oip.setNextObjectOID(objectInfo.getOid());
		//				// Here we are working in unconnected zone, so this
		//				// can be done without transaction: actually
		//				// write in database file
		//				updateNextObjectFieldOfObjectInfo(oip.getOid(), oip.getNextObjectOID(), false);
		//				objectInfo.setPreviousInstanceOID(oip.getOid());
		//				// Resets the class info oid: In some case,
		//				// (client // server) it may be -1.
		//				oip.setClassInfoId(classInfo.getId());
		//				// object info oip has been changed, we must put it
		//				// in the cache to turn this change available for current
		//				// transaction until the commit
		//				storageEngine.getSession(true).getCache().addObjectInfo(oip);
		//
		//			}
		//		}
		//		// always set the new last object oid and the number of objects
		//		classInfo.getUncommittedZoneInfo().last = objectInfo.getOid();
		//		classInfo.getUncommittedZoneInfo().increaseNbObjects();
		//		// Then updates the last info pointers of the class info
		//		// with this new created object
		//		// At this moment, the objectInfo.getHeader() do not have the
		//		// attribute ids.
		//		// but later in this code, the attributes will be set, so the class
		//		// info also will have them
		//		classInfo.setLastObjectInfoHeader(objectInfo.getHeader());
		//
		//		// // Saves the fact that something has changed in the class (number of
		//		// objects and/or last object oid)
		//		storageEngine.getSession(true).getMetaModel().addChangedClass(classInfo);
		//	}
		//
		//	/**
		//	 * Insert the object in the index
		//	 * 
		//	 * @param oid
		//	 *            The object id
		//	 * @param nnoi
		//	 *            The object meta representation
		//	 * @return The number of indexes
		//	 */
		//	private int manageIndexesForInsert(OID oid, NonNativeObjectInfo nnoi) {
		//		List indexes = nnoi.getClassInfo().getIndexes();
		//		ClassInfoIndex index = null;
		//		for (int i = 0; i < indexes.size(); i++) {
		//			index = (ClassInfoIndex) indexes.get(i);
		//			index.getBTree().insert(index.computeKey(nnoi), oid);
		//
		//			// Check consistency : index should have size equal to the class
		//			// info element number
		//			if (index.getBTree().getSize() != nnoi.getClassInfo().getNumberOfObjects()) {
		//				throw new ODBRuntimeException(Error.BTREE_SIZE_DIFFERS_FROM_CLASS_ELEMENT_NUMBER.addParameter(index.getBTree().getSize())
		//						.addParameter(nnoi.getClassInfo().getNumberOfObjects()));
		//			}
		//
		//		}
		//		return indexes.size();
		//	}
		//
		//	/**
		//	 * Insert the object in the index
		//	 * 
		//	 * @param oid
		//	 *            The object id
		//	 * @param nnoi
		//	 *            The object meta represenation
		//	 * @return The number of indexes
		//	 * @throws Exception
		//	 */
		//	protected int manageIndexesForDelete(OID oid, NonNativeObjectInfo nnoi) throws Exception {
		//		List indexes = nnoi.getClassInfo().getIndexes();
		//		ClassInfoIndex index = null;
		//
		//		for (int i = 0; i < indexes.size(); i++) {
		//			index = (ClassInfoIndex) indexes.get(i);
		//			// TODO manage collision!
		//			index.getBTree().delete(index.computeKey(nnoi), oid);
		//
		//			// Check consistency : index should have size equal to the class
		//			// info element number
		//			if (index.getBTree().getSize() != nnoi.getClassInfo().getNumberOfObjects()) {
		//				throw new ODBRuntimeException(Error.BTREE_SIZE_DIFFERS_FROM_CLASS_ELEMENT_NUMBER.addParameter(index.getBTree().getSize())
		//						.addParameter(nnoi.getClassInfo().getNumberOfObjects()));
		//			}
		//		}
		//
		//		return indexes.size();
		//	}
		//
		//	private int manageIndexesForUpdate(OID oid, NonNativeObjectInfo nnoi, NonNativeObjectInfo oldMetaRepresentation) throws Exception {
		//		// takes the indexes from the oldMetaRepresentation because noi comes
		//		// from the client and is not always
		//		// in sync with the server meta model (In Client Server mode)
		//		List indexes = oldMetaRepresentation.getClassInfo().getIndexes();
		//		ClassInfoIndex index = null;
		//		Comparable oldKey = null;
		//		Comparable newKey = null;
		//		for (int i = 0; i < indexes.size(); i++) {
		//			index = (ClassInfoIndex) indexes.get(i);
		//			oldKey = index.computeKey(oldMetaRepresentation);
		//			newKey = index.computeKey(nnoi);
		//			// Only update index if kyw has changed!
		//			if (oldKey.compareTo(newKey) != 0) {
		//				IBTree btree = index.getBTree();
		//				// TODO manage collision!
		//				Object old = btree.delete(oldKey, oid);
		//				// TODO check if old is equal to oldKey
		//				btree.insert(newKey, oid);
		//
		//				// Check consistency : index should have size equal to the class
		//				// info element number
		//				if (index.getBTree().getSize() != nnoi.getClassInfo().getNumberOfObjects()) {
		//					throw new ODBRuntimeException(Error.BTREE_SIZE_DIFFERS_FROM_CLASS_ELEMENT_NUMBER.addParameter(
		//							index.getBTree().getSize()).addParameter(nnoi.getClassInfo().getNumberOfObjects()));
		//				}
		//			}
		//		}
		//		return indexes.size();
		//	}
		//
		//	/*
		//	 * private long insertObject(long oid, AbstractObjectInfo aoi, boolean
		//	 * updatePointers) throws Exception { if (aoi.isNonNativeObject()) { return
		//	 * insertNonNativeObject(oid, (NonNativeObjectInfo) aoi, updatePointers); }
		//	 * if (aoi.isNative()) { return insertNativeObject((NativeObjectInfo) aoi); }
		//	 * 
		//	 * throw new
		//	 * ODBRuntimeException(Error.ABSTRACT_OBJECT_INFO_TYPE_NOT_SUPPORTED.addParameter(aoi.getClass().getName())); }
		//	 */
		//
		//	/**
		//	 * @param oid
		//	 *            The Oid of the object to be inserted
		//	 * @param nnoi
		//	 *            The object meta representation The object to be inserted in
		//	 *            the database
		//	 * @param isNewObject
		//	 *            To indicate if object is new
		//	 * @return The position of the inserted object
		//	 * @throws Exception
		//	 */
		//
		//	private OID insertNonNativeObject(OID oid, NonNativeObjectInfo nnoi, boolean isNewObject) throws Exception {
		//		try {
		//
		//			ClassInfo ci = nnoi.getClassInfo();
		//			Object object = nnoi.getObject();
		//
		//			// triggers
		//			triggerManager.manageInsertTriggerBefore(object);
		//
		//			// First check if object is already being inserted
		//			// This method returns -1 if object is not being inserted
		//			OID cachedOid = getSession().getCache().idOfInsertingObject(object);
		//			if (cachedOid != null) {
		//				return cachedOid;
		//			}
		//
		//			// Then checks if the class of this object already exist in the
		//			// meta model
		//			ci = storageEngine.getObjectWriter().addClass(ci, true);
		//			// Resets the ClassInfo in the objectInfo to be sure it contains all
		//			// updated class info data
		//			nnoi.setClassInfo(ci);
		//
		//			// Mark this object as being inserted. To manage cyclic relations
		//			// The oid may be equal to -1
		//			// Later in the process the cache will be updated with the right oid
		//			getSession().getCache().startInsertingObjectWithOid(object, oid, nnoi);
		//
		//			// false : do not write data in transaction. Data are always written
		//			// directly to disk. Pointers are written in transaction
		//			OID newOid = writeNonNativeObjectInfo(oid, nnoi, -1, false, isNewObject);
		//			if (newOid != StorageEngineConstant.NULL_OBJECT_ID) {
		//				getSession().getCache().addObject(newOid, object, nnoi.getHeader());
		//
		//				// triggers
		//				triggerManager.manageInsertTriggerAfter(object, newOid, nnoi.getHeader().getPosition());
		//			}
		//
		//			return newOid;
		//
		//		} finally {
		//			// This will be done by the mainStoreObject method
		//			// Context.getCache().endInsertingObject(object);
		//
		//		}
		//	}
		//
		//	/**
		//	 * 
		//	 * @param noi
		//	 *            The native object meta representation The object to be
		//	 *            inserted in the database
		//	 * @return The position of the inserted object
		//	 * @throws Exception
		//	 */
		//
		//	private long insertNativeObject(NativeObjectInfo noi) throws Exception {
		//		long writePosition = fsi.getAvailablePosition();
		//		fsi.setWritePosition(writePosition, true);
		//
		//		// true,false = update pointers,do not write in transaction, writes
		//		// directly to hard disk
		//		long position = writeNativeObjectInfo(noi, writePosition, true, false);
		//		return position;
		//	}
		//
		//	/**
		//	 * Store a meta representation of an object(already as meta
		//	 * representation)in ODBFactory database.
		//	 * 
		//	 * To detect if object must be updated or insert, we use the cache. To
		//	 * update an object, it must be first selected from the database. When an
		//	 * object is to be stored, if it exist in the cache, then it will be
		//	 * updated, else it will be inserted as a new object. If the object is null,
		//	 * the cache will be used to check if the meta representation is in the
		//	 * cache
		//	 * 
		//	 * @param oid
		//	 *            The oid of the object to be inserted/updates
		//	 * @param nnoi
		//	 *            The meta representation of an object
		//	 * @return The object position
		//	 * @throws IOException
		//	 */
		//	OID internalStoreObject(OID oid, NonNativeObjectInfo nnoi) throws Exception {
		//
		//		// first detects if we must perform an insert or an update
		//		// If object is in the cache, we must perform an update, else an insert
		//
		//		Object object = nnoi.getObject();
		//
		//		boolean mustUpdate = false;
		//		ICache cache = getSession().getCache();
		//
		//		if (object != null) {
		//			OID cacheOid = cache.idOfInsertingObject(object);
		//			if (cacheOid != null) {
		//				return cacheOid;
		//			}
		//
		//			// throw new ODBRuntimeException("Inserting meta representation of
		//			// an object without the object itself is not yet supported");
		//			mustUpdate = cache.existObject(object);
		//
		//		}
		//
		//		if (!mustUpdate) {
		//			mustUpdate = nnoi.getOid() != StorageEngineConstant.NULL_OBJECT_ID;
		//
		//		}
		//
		//		if (mustUpdate) {
		//			return manageUpdateNonNativeObjectInfo((NonNativeObjectInfo) nnoi, false);
		//		}
		//
		//		return insertNonNativeObject(oid, (NonNativeObjectInfo) nnoi, true);
		//	}
		//
		//	/**
		//	 * Store a meta representation of a native object(already as meta
		//	 * representation)in ODBFactory database. A Native object is an object that
		//	 * use native language type, String for example
		//	 * 
		//	 * To detect if object must be updated or insert, we use the cache. To
		//	 * update an object, it must be first selected from the database. When an
		//	 * object is to be stored, if it exist in the cache, then it will be
		//	 * updated, else it will be inserted as a new object. If the object is null,
		//	 * the cache will be used to check if the meta representation is in the
		//	 * cache
		//	 * 
		//	 * @param nnoi
		//	 *            The meta representation of an object
		//	 * @return The object position
		//	 * @throws IOException
		//	 */
		//	long internalStoreObject(NativeObjectInfo noi) throws Exception {
		//		return insertNativeObject(noi);
		//	}
		//
		//	public OID updateObject(AbstractObjectInfo aoi, boolean forceUpdate) throws Exception {
		//		if (aoi.isNonNativeObject()) {
		//			return manageUpdateNonNativeObjectInfo((NonNativeObjectInfo) aoi, forceUpdate);
		//		}
		//		if (aoi.isNative()) {
		//			return updateObject((NativeObjectInfo) aoi, forceUpdate);
		//		}
		//
		//		// TODO : here should use if then else
		//		throw new ODBRuntimeException(Error.ABSTRACT_OBJECT_INFO_TYPE_NOT_SUPPORTED.addParameter(aoi.getClass().getName()));
		//
		//	}
		//
		//	/**
		//	 * Updates an object.
		//	 * 
		//	 * <pre>
		//	 * Try to update in place. Only change what has changed. This is restricted to particular types (fixed size types). If in place update is 
		//	 * not possible, then deletes the current object and creates a new at the end of the database file and updates
		//	 * OID object position.
		//	 * 
		//	 * &#064;param nnoi The meta representation of the object to be updated
		//	 * &#064;param forceUpdate when true, no verification is done to check if update must be done.
		//	 * &#064;return The oid of the object, as a negative number
		//	 * &#064;throws IOException
		//	 * 
		//	 */
		//	protected OID manageUpdateNonNativeObjectInfo(NonNativeObjectInfo nnoi, boolean forceUpdate) throws Exception {
		//		boolean debugOn = Configuration.isDebugEnabled(LOG_ID);
		//		nbCallsToUpdate++;
		//		boolean hasObject = true;
		//		String message = null;
		//		Object object = nnoi.getObject();
		//		OID oid = nnoi.getOid();
		//
		//		if (object == null) {
		//			hasObject = false;
		//		}
		//		// When there is index,we must *always* load the old meta representation
		//		// to compute index keys
		//		boolean withIndex = !nnoi.getClassInfo().getIndexes().isEmpty();
		//
		//		NonNativeObjectInfo oldMetaRepresentation = null;
		//
		//		boolean objectHasChanged = false;
		//		IObjectInfoComparator objectComparator = new ObjectInfoComparator();
		//		try {
		//
		//			long positionBeforeWrite = fsi.getPosition();
		//
		//			ICache cache = getSession().getCache();
		//
		//			// Get header of the object (position, previous object position,
		//			// next
		//			// object position and class info position)
		//			// The header must be in the cache.
		//			ObjectInfoHeader header = cache.getObjectInfoHeaderFromOid(oid, true);
		//
		//			if (header == null) {
		//				throw new ODBRuntimeException(Error.UNEXPECTED_SITUATION.addParameter("Header is null in update"));
		//			}
		//
		//			if (header.getOid() == null) {
		//				throw new ODBRuntimeException(Error.INTERNAL_ERROR.addParameter("Header oid is null for oid " + oid));
		//			}
		//
		//			boolean objectIsInConnectedZone = cache.objectWithIdIsInCommitedZone(oid);
		//
		//			long currentPosition = header.getPosition();
		//
		//			// When using client server mode, we must re-read the position of
		//			// the object with oid. Because, another session may
		//			// have updated the object, and in this case, the position of the
		//			// object in the cache may be invalid
		//			// TODO It should be done only when the object has deleted or
		//			// updated by another session. Should chek this
		//			// Doing this with new objects (created in the current session, the
		//			// last committed
		//			// object position will be negative, in this case we must use the
		//			// currentPosition
		//			if (!isLocalMode) {
		//				long lastCommitedObjectPosition = idManager.getObjectPositionWithOid(oid, false);
		//				if (lastCommitedObjectPosition > 0) {
		//					currentPosition = lastCommitedObjectPosition;
		//				}
		//			}
		//
		//			// for client server
		//			if (nnoi.getPosition() == -1) {
		//				nnoi.getHeader().setPosition(currentPosition);
		//			}
		//
		//			if (currentPosition == -1) {
		//				throw new ODBRuntimeException(Error.INSTANCE_POSITION_IS_NEGATIVE.addParameter(currentPosition).addParameter(oid)
		//						.addParameter("In Object Info Header"));
		//			}
		//
		//			if (debugOn) {
		//				message = depthToSpaces() + "start updating object at " + currentPosition + ", oid=" + oid + " : "
		//						+ (nnoi != null ? nnoi.toString() : "null");
		//				DLogger.debug(message);
		//			}
		//
		//			if (hasObject) {
		//				// triggers
		//				triggerManager.manageUpdateTriggerBefore(null, object, oid);
		//			}
		//			// Use to control if the in place update is ok. The
		//			// ObjectInstrospector stores the number of changes
		//			// that were detected and here we try to apply them using in place
		//			// update.If at the end
		//			// of the in place update the number of applied changes is smaller
		//			// then the number
		//			// of detected changes, then in place update was not successfully,
		//			// we
		//			// must do a real update,
		//			// creating a object elsewhere :-(
		//			int nbAppliedChanges = 0;
		//			if (!forceUpdate) {
		//
		//				OID cachedOid = cache.idOfInsertingObject(object);
		//				if (cachedOid != null) {
		//					// The object is being inserted (must be a cyclic
		//					// reference), simply returns id id
		//					return cachedOid;
		//				}
		//
		//				// the nnoi (NonNativeObjectInfo is the meta representation of
		//				// the object to update
		//				// To know what must be upated we must get the meta
		//				// representation of this object before
		//				// The modification. Taking this 'old' meta representation from
		//				// the
		//				// cache does not resolve
		//				// : because cache is a reference to the real object and object
		//				// has been changed,
		//				// so the cache is pointing to the reference, that has changed!
		//				// This old meta representation must be re-read from the last
		//				// committed database
		//				// false, = returnInstance (java object) = false
		//				try {
		//					boolean useCache = !objectIsInConnectedZone;
		//					oldMetaRepresentation = objectReader.readNonNativeObjectInfoFromPosition(null, oid, currentPosition, useCache, false);
		//				} catch (ODBRuntimeException e) {
		//					throw new ODBRuntimeException(Error.INTERNAL_ERROR.addParameter("Error while reading old Object Info of oid " + oid
		//							+ " at pos " + currentPosition), e);
		//				}
		//
		//				// Set the object of the old meta to make the object comparator
		//				// understand, they are 2
		//				// meta representation of the same object
		//				// TODO , check if if is the best way to do
		//				oldMetaRepresentation.setObject(nnoi.getObject());
		//
		//				objectComparator = new ObjectInfoComparator();
		//				objectHasChanged = objectComparator.hasChanged(oldMetaRepresentation, nnoi);
		//
		//				if (!objectHasChanged) {
		//					fsi.setWritePosition(positionBeforeWrite, true);
		//
		//					if (debugOn) {
		//						DLogger.debug(depthToSpaces() + " => updateObject : Object with oid " + oid + " is unchanged - doing nothing");
		//					}
		//					return oid;
		//				}
		//
		//				if (debugOn) {
		//					DLogger.debug(depthToSpaces() + "\tmax recursion level is " + objectComparator.getMaxObjectRecursionLevel());
		//					DLogger.debug(depthToSpaces() + "\tattribute actions are : " + objectComparator.getChangedAttributeActions());
		//					DLogger.debug(depthToSpaces() + "\tnew objects are : " + objectComparator.getNewObjects());
		//				}
		//
		//				if (Configuration.inPlaceUpdate() && objectComparator.supportInPlaceUpdate()) {
		//					nbAppliedChanges = manageInPlaceUpdate(objectComparator, object, oid, header, cache, objectIsInConnectedZone);
		//				}
		//				// if number of applied changes is equal to the number of
		//				// detected change
		//				if (nbAppliedChanges == objectComparator.getNbChanges()) {
		//					nbInPlaceUpdates++;
		//					return oid;
		//				}
		//
		//			}
		//
		//			if (debugOn) {
		//				DLogger.debug(depthToSpaces() + "Update in place not performed -> normal update");
		//			}
		//			/*
		//			// Try to update only actually updated objects
		//			if(objectComparator.getNbChanges()==objectComparator.getChangedAttributeActions().size()){
		//				int nbChanges = objectComparator.getChangedAttributeActions().size();
		//				ChangedNativeAttributeAction cnaa = null;
		//				for(int i=0;i<nbChanges;i++){
		//					cnaa = (ChangedNativeAttributeAction) objectComparator.getChangedAttributeActions().get(i);
		//					if(debugOn){
		//						DLogger.debug(depthToSpaces() + "Updating attribute object with oid "+cnaa.getOldNnoi().getOid());
		//					}
		//					updateNonNativeObjectInfo(cnaa.getNewNoi(), cnaa.getOldNnoi(), withIndex, hasObject, cache, object);					
		//				}
		//				return oid;
		//			}
		//			*/
		//			
		//			return updateNonNativeObjectInfo(nnoi, oldMetaRepresentation, withIndex, hasObject, cache, object);
		//		} finally {
		//		}
		//	}
		//
		//	protected OID updateNonNativeObjectInfo(NonNativeObjectInfo nnoi, NonNativeObjectInfo oldMetaRepresentation,
		//			boolean withIndex, boolean hasObject, ICache cache, Object object)
		//			throws Exception {
		//
		//		OID oid = nnoi.getOid();
		//		ObjectInfoHeader header = oldMetaRepresentation.getHeader();
		//		long currentPosition = oldMetaRepresentation.getHeader().getPosition();
		//		boolean objectIsInConnectedZone = cache.objectWithIdIsInCommitedZone(oid);
		//		boolean debugOn = Configuration.isDebugEnabled(LOG_ID);
		//		try {
		//
		//			// If we reach this update, In Place Update was not possible. Do a
		//			// normal update. Deletes the
		//			// current object and creates a new one
		//
		//			if (oldMetaRepresentation == null && withIndex) {
		//				// We must load old meta representation to be able to compute
		//				// old index key to update index
		//				oldMetaRepresentation = objectReader.readNonNativeObjectInfoFromPosition(null, oid, currentPosition, false, false);
		//			}
		//			nbNormalUpdates++;
		//			if (hasObject) {
		//				cache.startInsertingObjectWithOid(object, oid, nnoi);
		//			}
		//
		//			// gets class info from in memory meta model
		//			ClassInfo ci = storageEngine.getSession(true).getMetaModel().getClassInfoFromId(header.getClassInfoId());
		//
		//			if (hasObject) {
		//				// removes the object from the cache
		//				// cache.removeObjectWithOid(oid, object);
		//				cache.endInsertingObject(object);
		//			}
		//
		//			OID previousObjectOID = header.getPreviousObjectOID();
		//			OID nextObjectOid = header.getNextObjectOID();
		//
		//			if (debugOn) {
		//				DLogger.debug(depthToSpaces() + "Updating object " + nnoi.toString());
		//				DLogger.debug(depthToSpaces() + "position =  " + currentPosition + " | prev instance = " + previousObjectOID
		//						+ " | next instance = " + nextObjectOid);
		//			}
		//
		//			nnoi.setPreviousInstanceOID(previousObjectOID);
		//			nnoi.setNextObjectOID(nextObjectOid);
		//
		//			// Mark the block of current object as deleted
		//			markAsDeleted(currentPosition, oid, objectIsInConnectedZone);
		//
		//			// Creates the new object
		//			oid = insertNonNativeObject(oid, nnoi, false);
		//			// This position after write must be call just after the insert!!
		//			long positionAfterWrite = fsi.getPosition();
		//			// Get the position of the new inserted object - it is in a
		//			// temporary cache of the id manager
		//			long newObjectPosition = idManager.getObjectPositionWithOid(oid, true);
		//
		//			header.setPosition(newObjectPosition);
		//
		//			if (hasObject) {
		//				// update cache
		//				cache.addObject(oid, object, header);
		//			}
		//
		//			fsi.setWritePosition(positionAfterWrite, true);
		//
		//			return oid;
		//		} catch (Exception e) {
		//			String message = depthToSpaces() + "Error updating object " + nnoi.toString() + " : " + e.getMessage();
		//			DLogger.error(message);
		//			throw e;
		//		} finally {
		//			if (withIndex) {
		//				manageIndexesForUpdate(oid, nnoi, oldMetaRepresentation);
		//			}
		//
		//			if (hasObject) {
		//				// triggers
		//				triggerManager.manageUpdateTriggerAfter(null, object, oid);
		//			}
		//			
		//			if (debugOn) {
		//				DLogger.debug(depthToSpaces() + "end updating object with oid=" + oid + " at pos " + nnoi.getPosition() + " => "
		//						+ nnoi.toString());
		//			}
		//		}
		//	}
		//
		//	protected ObjectInfoHeader getObjectInfoHeader(OID oid, ICache cache) throws IOException {
		//		ObjectInfoHeader oih = cache.getObjectInfoHeaderFromOid(oid, false);
		//		// If object is not in the cache, then read the header from the file
		//		if (oih == null) {
		//			oih = objectReader.readObjectInfoHeaderFromOid(oid, false);
		//		}
		//		return oih;
		//	}
		//
		//	protected ObjectInfoHeader updateNextObjectPreviousPointersInCache(OID nextObjectOID, OID previousObjectOID, ICache cache)
		//			throws IOException {
		//		ObjectInfoHeader oip = cache.getObjectInfoHeaderFromOid(nextObjectOID, false);
		//		// If object is not in the cache, then read the header from the file
		//		if (oip == null) {
		//			oip = objectReader.readObjectInfoHeaderFromOid(nextObjectOID, false);
		//			cache.addObjectInfo(oip);
		//		}
		//		oip.setPreviousObjectOID(previousObjectOID);
		//
		//		return oip;
		//	}
		//
		//	protected ObjectInfoHeader updatePreviousObjectNextPointersInCache(OID nextObjectOID, OID previousObjectOID, ICache cache)
		//			throws IOException {
		//		ObjectInfoHeader oip = cache.getObjectInfoHeaderFromOid(previousObjectOID, false);
		//		// If object is not in the cache, then read the header from the file
		//		if (oip == null) {
		//			oip = objectReader.readObjectInfoHeaderFromOid(previousObjectOID, false);
		//			cache.addObjectInfo(oip);
		//		}
		//		oip.setNextObjectOID(nextObjectOID);
		//
		//		return oip;
		//	}
		//
		//	/**
		//	 * Manage in place update. Just write the value at the exact position if
		//	 * possible.
		//	 * 
		//	 * @param objectComparator
		//	 *            Contains all infos about differences between all version
		//	 *            objects and new version
		//	 * @param object
		//	 *            The object being modified (new version)
		//	 * @param oid
		//	 *            The oid of the object being modified
		//	 * @param header
		//	 *            The header of the object meta representation (Comes from the
		//	 *            cache)
		//	 * @param cache
		//	 *            The cache it self
		//	 * @param objectInInConnectedZone
		//	 *            A boolean value to indicate if object is in connected zone. I
		//	 *            true, change must be made in transaction. If false, changes
		//	 *            can be made in the database file directly.
		//	 * @return The number of in place update successfully executed
		//	 * @throws Exception
		//	 */
		//	private int manageInPlaceUpdate(IObjectInfoComparator objectComparator, Object object, OID oid, ObjectInfoHeader header, ICache cache,
		//			boolean objectIsInConnectedZone) throws Exception {
		//		boolean canUpdateInPlace = true;
		//		// If object is is connected zone, changes must be done in transaction,
		//		// if not in connected zone, changes can be made out of
		//		// transaction, directly to the database
		//		boolean writeInTransaction = objectIsInConnectedZone;
		//		int nbAppliedChanges = 0;
		//		// if 0, only direct attribute have been changed
		//		// if (objectComparator.getMaxObjectRecursionLevel() == 0) {
		//		// if some direct native attribute have changed
		//		if (!objectComparator.getChangedAttributeActions().isEmpty()) {
		//			ChangedNativeAttributeAction caa = null;
		//
		//			// Check if in place update is possible
		//			List actions = objectComparator.getChangedAttributeActions();
		//			for (int i = 0; i < actions.size(); i++) {
		//				if (actions.get(i) instanceof ChangedNativeAttributeAction) {
		//					caa = (ChangedNativeAttributeAction) actions.get(i);
		//					if (caa.reallyCantDoInPlaceUpdate()) {
		//						canUpdateInPlace = false;
		//						break;
		//					}
		//
		//					if (!caa.inPlaceUpdateIsGuaranteed()) {
		//						if (caa.isString() && caa.getUpdatePosition() != StorageEngineConstant.NULL_OBJECT_POSITION) {
		//							long position = safeOverWriteAtomicNativeObject(caa.getUpdatePosition(), (AtomicNativeObjectInfo) caa
		//									.getNoiWithNewValue(), writeInTransaction);
		//							canUpdateInPlace = position != -1;
		//							if (!canUpdateInPlace) {
		//								break;
		//							}
		//						} else {
		//							canUpdateInPlace = false;
		//							break;
		//						}
		//					} else {
		//						fsi.setWritePosition(caa.getUpdatePosition(), true);
		//						writeAtomicNativeObject((AtomicNativeObjectInfo) caa.getNoiWithNewValue(), writeInTransaction);
		//					}
		//				} else if (actions.get(i) instanceof ChangedObjectReferenceAttributeAction) {
		//					ChangedObjectReferenceAttributeAction coraa = (ChangedObjectReferenceAttributeAction) actions.get(i);
		//					updateObjectReference(coraa.getUpdatePosition(), coraa.getNewId(), writeInTransaction);
		//				}
		//				nbAppliedChanges++;
		//			}
		//			if (canUpdateInPlace) {
		//				if (Configuration.isDebugEnabled(LOG_ID)) {
		//					DLogger.debug(depthToSpaces() + "Sucessfull in place updating");
		//				}
		//				/**
		//				 * Here we do not need to remove from the cache and add it
		//				 * again. Just let it. cache.removeObjectWithOid(oid, object);
		//				 * cache.addObject(oid, object, header);
		//				 */
		//			}
		//		}
		//
		//		// if canUpdateInplace is false, a full update (writing
		//		// object elsewhere) is necessary so
		//		// there is no need to try to update object references.
		//		if (canUpdateInPlace) {
		//			NewNonNativeObjectAction nnnoa = null;
		//			// For non native attribute that have been replaced!
		//			for (int i = 0; i < objectComparator.getNewObjectMetaRepresentations().size(); i++) {
		//				// to avoid stackOverFlow, check if the object is
		//				// already beeing inserted
		//				nnnoa = objectComparator.getNewObjectMetaRepresentation(i);
		//				if (cache.idOfInsertingObject(nnnoa) == null) {
		//					OID ooid = nnnoa.getNnoi().getOid();
		//					// If Meta representation have an id == null, then
		//					// this is a new object
		//					// it must be inserted, else just update
		//					// reference
		//					if (ooid == null) {
		//						ooid = insertNonNativeObject(null, nnnoa.getNnoi(), true);
		//					}
		//					updateObjectReference(nnnoa.getUpdatePosition(), ooid, writeInTransaction);
		//					nbAppliedChanges++;
		//				}
		//			}
		//			SetAttributeToNullAction satna = null;
		//			// For attribute that have been set to null
		//			for (int i = 0; i < objectComparator.getAttributeToSetToNull().size(); i++) {
		//				satna = (SetAttributeToNullAction) objectComparator.getAttributeToSetToNull().get(i);
		//				updateObjectReference(satna.getUpdatePosition(), StorageEngineConstant.NULL_OBJECT_ID, writeInTransaction);
		//				nbAppliedChanges++;
		//			}
		//			ArrayModifyElement ame = null;
		//			// For attribute that have been set to null
		//			for (int i = 0; i < objectComparator.getArrayChanges().size(); i++) {
		//				ame = (ArrayModifyElement) objectComparator.getArrayChanges().get(i);
		//				if (!ame.supportInPlaceUpdate()) {
		//					break;
		//				}
		//				fsi.setReadPosition(ame.getArrayPositionDefinition());
		//				long arrayPosition = fsi.readLong();
		//				// If we reach this line,the ArrayModifyElement
		//				// suuports In Place Update so it must be a Native
		//				// Object Info!
		//				// The cast is safe :-)
		//				updateArrayElement(arrayPosition, ame.getArrayElementIndexToChange(), (NativeObjectInfo) ame.getNewValue(),
		//						writeInTransaction);
		//				nbAppliedChanges++;
		//			}
		//		}
		//		// }// only direct attribute have been changed
		//		/*
		//		 * else { // check if objects of other recursion levels have been
		//		 * changed! for (int i = 0; i <
		//		 * objectComparator.getChangedObjectMetaRepresentations().size(); i++) { //
		//		 * to avoid stackOverFlow, check if the object is // already beeing
		//		 * inserted // olivier:19/10/2006, changed from != to == if
		//		 * (cache.idOfInsertingObject(objectComparator.getChangedObjectMetaRepresentation(i)) ==
		//		 * -1) {
		//		 * updateObject(objectComparator.getChangedObjectMetaRepresentation(i),
		//		 * false); } } }
		//		 */
		//		return nbAppliedChanges;
		//
		//	}
		//
		//	private boolean canDoInPlaceUpdate(long updatePosition, String value) throws IOException {
		//		fsi.setReadPosition(updatePosition + StorageEngineConstant.NATIVE_OBJECT_OFFSET_DATA_AREA);
		//		int totalSize = fsi.readInt("String total size");
		//		int stringNumberOfBytes = byteArrayConverter.getNumberOfBytesOfAString(value);
		//		// Checks if there is enough space to store this new string in place
		//		return totalSize >= stringNumberOfBytes;
		//	}
		//
		//	private String depthToSpaces() {
		//		StringBuffer buffer = new StringBuffer();
		//		for (int i = 0; i < currentDepth; i++) {
		//			buffer.append("  ");
		//		}
		//		return buffer.toString();
		//	}
		//
		//	private void writeBlockSizeAt(long writePosition, int blockSize, boolean writeInTransaction, Object object) throws IOException {
		//		if (blockSize < 0) {
		//			throw new ODBRuntimeException(Error.NEGATIVE_BLOCK_SIZE.addParameter(writePosition).addParameter(blockSize).addParameter(
		//					object.toString()));
		//		}
		//
		//		long currentPosition = fsi.getPosition();
		//		fsi.setWritePosition(writePosition, writeInTransaction);
		//		fsi.writeInt(blockSize, writeInTransaction, "block size");
		//		// goes back where we were
		//		fsi.setWritePosition(currentPosition, writeInTransaction);
		//
		//	}
		//
		//	/**
		//	 * TODO check if we should pass the position instead of requesting if to fsi
		//	 * 
		//	 * <pre>
		//	 *                          Write a collection to the database
		//	 *                          
		//	 *                          This is done by writing the number of element s and then the position of all elements.
		//	 *                          
		//	 *                          Example : a list with two string element : 'ola' and 'chico'
		//	 *                          
		//	 *                          write 2 (as an int) : the number of elements
		//	 *                          write two times 0 (as long) to reserve the space for the elements positions
		//	 *                          
		//	 *                          then write the string 'ola', and keeps its position in the 'positions' array of long 
		//	 *                          then write the string 'chico' and keeps its position in the 'positions' array of long
		//	 *                          
		//	 *                          Then write back all the positions (in this case , 2 positions) after the size of the collection
		//	 *                          &lt;pre&gt;
		//	 *                          &#064;param coi
		//	 *                          &#064;param writeInTransaction
		//	 *                          &#064;throws IOException 
		//	 * 
		//	 */
		//	private long writeCollection(CollectionObjectInfo coi, boolean writeInTransaction) throws Exception {
		//		long firstObjectPosition = 0;
		//		long[] attributeIdentifications;
		//		long startPosition = fsi.getPosition();
		//
		//		writeNativeObjectHeader(coi.getOdbTypeId(), coi.isNull(), BlockTypes.BLOCK_TYPE_COLLECTION_OBJECT, writeInTransaction);
		//
		//		if (coi.isNull()) {
		//			return startPosition;
		//		}
		//		Collection collection = coi.getCollection();
		//		int collectionSize = collection.size();
		//		Iterator iterator = collection.iterator();
		//
		//		// write the real type of the collection
		//		fsi.writeString(coi.getRealCollectionClassName(), writeInTransaction);
		//
		//		// write the size of the collection
		//		fsi.writeInt(collectionSize, writeInTransaction, "collection size");
		//		// build a n array to store all element positions
		//		attributeIdentifications = new long[collectionSize];
		//		// Gets the current position, to know later where to put the
		//		// references
		//		firstObjectPosition = fsi.getPosition();
		//
		//		// reserve space for object positions : write 'collectionSize' long
		//		// with zero to store each object position
		//		for (int i = 0; i < collectionSize; i++) {
		//			fsi.writeLong(0, writeInTransaction, "collection element pos ", DefaultWriteAction.DATA_WRITE_ACTION);
		//		}
		//		int currentElement = 0;
		//		AbstractObjectInfo element = null;
		//		while (iterator.hasNext()) {
		//			element = (AbstractObjectInfo) iterator.next();
		//			attributeIdentifications[currentElement] = internalStoreObjectWrapper(element);
		//			currentElement++;
		//		}
		//
		//		long positionAfterWrite = fsi.getPosition();
		//		// now that all objects have been stored, sets their position in the
		//		// space that have been reserved
		//		fsi.setWritePosition(firstObjectPosition, writeInTransaction);
		//		for (int i = 0; i < collectionSize; i++) {
		//			fsi.writeLong(attributeIdentifications[i], writeInTransaction, "collection element real pos ", DefaultWriteAction.DATA_WRITE_ACTION);
		//		}
		//		// Goes back to the end of the array
		//		fsi.setWritePosition(positionAfterWrite, writeInTransaction);
		//
		//		return startPosition;
		//	}
		//
		//	/**
		//	 * <pre>
		//	 *                          Write an array to the database
		//	 *                          
		//	 *                          This is done by writing :
		//	 *                          - the array type : array
		//	 *                          - the array element type (String if it os a String [])
		//	 *                          - the position of the non native type, if element are non java / C# native
		//	 *                          - the number of element s and then the position of all elements.
		//	 *                          
		//	 *                          Example : an array with two string element : 'ola' and 'chico'
		//	 *                          write 22 : array
		//	 *                          write  20 : array of STRING
		//	 *                          write 0 : it is a java native object
		//	 *                          write 2 (as an int) : the number of elements
		//	 *                          write two times 0 (as long) to reserve the space for the elements positions
		//	 *                          
		//	 *                          then write the string 'ola', and keeps its position in the 'positions' array of long 
		//	 *                          then write the string 'chico' and keeps its position in the 'positions' array of long
		//	 *                          
		//	 *                          Then write back all the positions (in this case , 2 positions) after the size of the array
		//	 *                          
		//	 *                          
		//	 *                          Example : an array with two User element : user1 and user2
		//	 *                          write 22 : array
		//	 *                          write  23 : array of NON NATIVE Objects
		//	 *                          write 251 : if 250 is the position of the user class info in database
		//	 *                          write 2 (as an int) : the number of elements
		//	 *                          write two times 0 (as long) to reserve the space for the elements positions
		//	 *                          
		//	 *                          then write the user user1, and keeps its position in the 'positions' array of long 
		//	 *                          then write the user user2 and keeps its position in the 'positions' array of long
		//	 *                          &lt;pre&gt;
		//	 *                          &#064;param object
		//	 *                          &#064;param odbType
		//	 *                          &#064;param position
		//	 *                          &#064;param writeInTransaction
		//	 *                          &#064;throws IOException 
		//	 * 
		//	 */
		//	private long writeArray(ArrayObjectInfo aoi, boolean writeInTransaction) throws Exception {
		//		long firstObjectPosition = 0;
		//		long[] attributeIdentifications;
		//		long startPosition = fsi.getPosition();
		//
		//		writeNativeObjectHeader(aoi.getOdbTypeId(), aoi.isNull(), BlockTypes.BLOCK_TYPE_ARRAY_OBJECT, writeInTransaction);
		//
		//		if (aoi.isNull()) {
		//			return startPosition;
		//		}
		//
		//		Object[] array = aoi.getArray();
		//		int arraySize = array.length;
		//
		//		// Writes the fact that it is an array
		//		fsi.writeString(aoi.getRealArrayComponentClassName(), writeInTransaction);
		//		/*
		//		 * // Write the java natuive type of teh elements of the array,
		//		 * NON_NATIVE // if not java native
		//		 * fsi.writeInt(aoi.getOdbType().getSubType().getId(),
		//		 * writeInTransaction, "native array type id"); if
		//		 * (aoi.getOdbType().getSubType() == ODBType.NON_NATIVE) {
		//		 * fsi.writeLong(storageEngine.getMetaModel().getClassInfo(aoi.getOdbType().getSubType().getName()).getPosition(),
		//		 * writeInTransaction, "non native array class position"); }
		//		 */
		//		// write the size of the array
		//		fsi.writeInt(arraySize, writeInTransaction, "array size");
		//		// build a n array to store all element positions
		//		attributeIdentifications = new long[arraySize];
		//		// Gets the current position, to know later where to put the
		//		// references
		//		firstObjectPosition = fsi.getPosition();
		//
		//		// reserve space for object positions : write 'arraySize' long
		//		// with zero to store each object position
		//		for (int i = 0; i < arraySize; i++) {
		//			fsi.writeLong(0, writeInTransaction, "array element pos ", DefaultWriteAction.DATA_WRITE_ACTION);
		//		}
		//
		//		AbstractObjectInfo element = null;
		//		for (int i = 0; i < arraySize; i++) {
		//			element = (AbstractObjectInfo) array[i];
		//
		//			if (element == null || element.isNull()) {
		//				// TODO Check this
		//				attributeIdentifications[i] = StorageEngineConstant.NULL_OBJECT_ID_ID;
		//				continue;
		//			}
		//			attributeIdentifications[i] = internalStoreObjectWrapper(element);
		//		}
		//
		//		long positionAfterWrite = fsi.getPosition();
		//		// now that all objects have been stored, sets their position in the
		//		// space that have been reserved
		//		fsi.setWritePosition(firstObjectPosition, writeInTransaction);
		//		for (int i = 0; i < arraySize; i++) {
		//			fsi.writeLong(attributeIdentifications[i], writeInTransaction, "array real element pos", DefaultWriteAction.DATA_WRITE_ACTION);
		//		}
		//		// Gos back to the end of the array
		//		fsi.setWritePosition(positionAfterWrite, writeInTransaction);
		//
		//		return startPosition;
		//	}
		//
		//	/**
		//	 * <pre>
		//	 *                          Write a map to the database
		//	 *                          
		//	 *                          This is done by writing the number of element s and then the key and value pair of all elements.
		//	 *                          
		//	 *                          Example : a map with two string element : '1/olivier' and '2/chico'
		//	 *                          
		//	 *                          write 2 (as an int) : the number of elements
		//	 *                          write 4 times 0 (as long) to reserve the space for the elements positions
		//	 *                          
		//	 *                          then write the object '1' and 'olivier', and keeps the two posiitons in the 'positions' array of long 
		//	 *                          then write the object '2' and the string chico' and keep the two position in the 'positions' array of long
		//	 *                          
		//	 *                          Then write back all the positions (in this case , 4 positions) after the size of the map
		//	 *                          
		//	 *                          &#064;param object
		//	 *                          &#064;param writeInTransaction To specify if these writes must be done in or out of a transaction
		//	 *                          &#064;throws IOException 
		//	 * 
		//	 */
		//	private long writeMap(MapObjectInfo moi, boolean writeInTransaction) throws Exception {
		//		long firstObjectPosition = 0;
		//		long[] positions;
		//		long startPosition = fsi.getPosition();
		//
		//		writeNativeObjectHeader(moi.getOdbTypeId(), moi.isNull(), BlockTypes.BLOCK_TYPE_MAP_OBJECT, writeInTransaction);
		//
		//		if (moi.isNull()) {
		//			return startPosition;
		//		}
		//
		//		Map map = moi.getMap();
		//		int mapSize = map.size();
		//		Iterator keys = map.keySet().iterator();
		//
		//		// write the map class
		//		fsi.writeString(moi.getRealMapClassName(), writeInTransaction);
		//		// write the size of the map
		//		fsi.writeInt(mapSize, writeInTransaction, "map size");
		//
		//		// build a n array to store all element positions
		//		positions = new long[mapSize * 2];
		//		// Gets the current position, to know later where to put the
		//		// references
		//		firstObjectPosition = fsi.getPosition();
		//
		//		// reserve space for object positions : write 'mapSize*2' long
		//		// with zero to store each object position
		//		for (int i = 0; i < mapSize * 2; i++) {
		//			fsi.writeLong(0, writeInTransaction, "map element pos", DefaultWriteAction.DATA_WRITE_ACTION);
		//		}
		//		int currentElement = 0;
		//		while (keys.hasNext()) {
		//			Object key = keys.next();
		//			Object value = map.get(key);
		//
		//			ODBType keyType = ODBType.getFromClass(key.getClass());
		//			ODBType valueType = ODBType.getFromClass(value.getClass());
		//
		//			positions[currentElement++] = internalStoreObjectWrapper((AbstractObjectInfo) key);
		//			positions[currentElement++] = internalStoreObjectWrapper((AbstractObjectInfo) value);
		//		}
		//
		//		long positionAfterWrite = fsi.getPosition();
		//		// now that all objects have been stored, sets their position in the
		//		// space that have been reserved
		//		fsi.setWritePosition(firstObjectPosition, writeInTransaction);
		//		for (int i = 0; i < mapSize * 2; i++) {
		//			fsi.writeLong(positions[i], writeInTransaction, "map real element pos", DefaultWriteAction.DATA_WRITE_ACTION);
		//		}
		//		// Gos back to the end of the array
		//		fsi.setWritePosition(positionAfterWrite, writeInTransaction);
		//
		//		return startPosition;
		//	}
		//
		//	/**
		//	 * This method is used to store the object : natibe or non native and return
		//	 * a number : - The position of the object if it is a native object - The
		//	 * oid (as a negative number) if it is a non native object
		//	 * 
		//	 * @param aoi
		//	 * @return
		//	 * @throws Exception
		//	 */
		//	private long internalStoreObjectWrapper(AbstractObjectInfo aoi) throws Exception {
		//		if (aoi.isNative()) {
		//			return internalStoreObject((NativeObjectInfo) aoi);
		//		}
		//		if (aoi.isNonNativeObject()) {
		//			// OID oid = internalStoreObject(null, (NonNativeObjectInfo) aoi);
		//			NonNativeObjectInfo nnoi = (NonNativeObjectInfo) aoi;
		//			return -nnoi.getOid().getObjectId();
		//		}
		//
		//		// Object references are references to object already stored.
		//		// But in the case of map, the reference can appear before the real
		//		// object (as order may change)
		//		// If objectReference.getOid() is null, it is the case. In this case,
		//		// We take the object being referenced and stores it directly.
		//		ObjectReference objectReference = (ObjectReference) aoi;
		//		if (objectReference.getOid() == null) {
		//			OID oid = internalStoreObject(null, (NonNativeObjectInfo) objectReference.getNnoi());
		//			return -oid.getObjectId();
		//		}
		//
		//		return -objectReference.getOid().getObjectId();
		//	}
		//
		//	protected void writeNullNativeObjectHeader(int OdbTypeId, boolean writeInTransaction) throws IOException {
		//		writeNativeObjectHeader(OdbTypeId, true, BlockTypes.BLOCK_TYPE_NATIVE_NULL_OBJECT, writeInTransaction);
		//	}
		//
		//	protected void writeNonNativeNullObjectHeader(OID classInfoId, boolean writeInTransaction) throws IOException {
		//		// Block size
		//		fsi.writeInt(NON_NATIVE_HEADER_BLOCK_SIZE, writeInTransaction, "block size");
		//		// Block type
		//		fsi.writeByte(BlockTypes.BLOCK_TYPE_NON_NATIVE_NULL_OBJECT, writeInTransaction);
		//		// class info id
		//		fsi.writeLong(classInfoId.getObjectId(), writeInTransaction, "null non native obj class info position",
		//				DefaultWriteAction.DATA_WRITE_ACTION);
		//	}
		//
		//	/**
		//	 * Write the header of a native attribute
		//	 * 
		//	 * @param odbTypeId
		//	 * @param isNull
		//	 * @param writeDataInTransaction
		//	 * @throws IOException
		//	 */
		//	protected void writeNativeObjectHeader(int odbTypeId, boolean isNull, byte blockType, boolean writeDataInTransaction)
		//			throws IOException {
		//
		//		// Block size
		//		fsi.writeInt(NATIVE_HEADER_BLOCK_SIZE, writeDataInTransaction, "native block header block size");
		//		// Block type
		//		fsi.writeByte(blockType, writeDataInTransaction, "native block type");
		//		// ODBFactory Object type
		//		fsi.writeInt(odbTypeId, writeDataInTransaction, "native block odb type id");
		//		// write a boolean value to indicate if object is null
		//		fsi.writeBoolean(isNull, writeDataInTransaction);
		//	}
		//
		//	public long safeOverWriteAtomicNativeObject(long position, AtomicNativeObjectInfo newAnoi, boolean writeInTransaction)
		//			throws NumberFormatException, IOException {
		//
		//		// If the attribute an a non fix ize, check if this write is safe
		//		if (ODBType.hasFixSize(newAnoi.getOdbTypeId())) {
		//			fsi.setWritePosition(position, writeInTransaction);
		//			return writeAtomicNativeObject(newAnoi, writeInTransaction);
		//		}
		//		if (ODBType.isStringOrBigDicemalOrBigInteger(newAnoi.getOdbTypeId())) {
		//			fsi.setReadPosition(position + StorageEngineConstant.NATIVE_OBJECT_OFFSET_DATA_AREA);
		//			int totalSize = fsi.readInt("String total size");
		//			int stringNumberOfBytes = byteArrayConverter.getNumberOfBytesOfAString(newAnoi.getObject().toString());
		//			// Checks if there is enough space to store this new string in place
		//			boolean canUpdate = totalSize >= stringNumberOfBytes;
		//			if (canUpdate) {
		//				fsi.setWritePosition(position, writeInTransaction);
		//				return writeAtomicNativeObject(newAnoi, writeInTransaction, totalSize);
		//			}
		//		}
		//		return -1;
		//	}
		//
		//	/**
		//	 * Writes a natibve attribute
		//	 * 
		//	 * @param anoi
		//	 * @param writeInTransaction
		//	 *            To specify if data must be written in the transaction or
		//	 *            directly to database file
		//	 * @return The object position
		//	 * @throws NumberFormatException
		//	 * @throws IOException
		//	 * 
		//	 * TODO the block is set to 0
		//	 */
		//	public long writeAtomicNativeObject(AtomicNativeObjectInfo anoi, boolean writeInTransaction) throws NumberFormatException, IOException {
		//		return writeAtomicNativeObject(anoi, writeInTransaction, -1);
		//	}
		//
		//	public long writeAtomicNativeObject(AtomicNativeObjectInfo anoi, boolean writeInTransaction, int totalSpaceIfString)
		//			throws NumberFormatException, IOException {
		//		long startPosition = fsi.getPosition();
		//		int odbTypeId = anoi.getOdbTypeId();
		//
		//		writeNativeObjectHeader(odbTypeId, anoi.isNull(), BlockTypes.BLOCK_TYPE_NATIVE_OBJECT, writeInTransaction);
		//
		//		if (anoi.isNull()) {
		//			// Even if object is null, reserve space for to simplify/enable in
		//			// place update
		//			fsi.ensureSpaceFor(anoi.getOdbType());
		//			return startPosition;
		//		}
		//
		//		Object object = anoi.getObject();
		//
		//		switch (odbTypeId) {
		//		case ODBType.BYTE_ID:
		//		case ODBType.NATIVE_BYTE_ID:
		//			fsi.writeByte(((Byte) object).byteValue(), writeInTransaction);
		//			break;
		//		case ODBType.BOOLEAN_ID:
		//		case ODBType.NATIVE_BOOLEAN_ID:
		//			fsi.writeBoolean(((Boolean) object).booleanValue(), writeInTransaction);
		//			fsi.writeBoolean(((Boolean) object).booleanValue(), writeInTransaction);
		//			break;
		//		case ODBType.CHARACTER_ID:
		//			fsi.writeChar(((Character) object).charValue(), writeInTransaction);
		//			break;
		//		case ODBType.NATIVE_CHAR_ID:
		//			fsi.writeChar(object.toString().charAt(0), writeInTransaction);
		//			break;
		//		case ODBType.FLOAT_ID:
		//		case ODBType.NATIVE_FLOAT_ID:
		//			fsi.writeFloat(((Float) object).floatValue(), writeInTransaction);
		//			break;
		//		case ODBType.DOUBLE_ID:
		//		case ODBType.NATIVE_DOUBLE_ID:
		//			fsi.writeDouble(((Double) object).doubleValue(), writeInTransaction);
		//			break;
		//		case ODBType.INTEGER_ID:
		//		case ODBType.NATIVE_INT_ID:
		//			fsi.writeInt(((Integer) object).intValue(), writeInTransaction, "native attr");
		//			break;
		//		case ODBType.LONG_ID:
		//		case ODBType.NATIVE_LONG_ID:
		//			fsi.writeLong(((Long) object).longValue(), writeInTransaction, "native attr", DefaultWriteAction.DATA_WRITE_ACTION);
		//			break;
		//		case ODBType.SHORT_ID:
		//		case ODBType.NATIVE_SHORT_ID:
		//			fsi.writeShort(((Short) object).shortValue(), writeInTransaction);
		//			break;
		//		case ODBType.BIG_DECIMAL_ID:
		//			fsi.writeBigDecimal((BigDecimal) object, writeInTransaction);
		//			break;
		//		case ODBType.BIG_INTEGER_ID:
		//			fsi.writeBigInteger((BigInteger) object, writeInTransaction);
		//			break;
		//		case ODBType.DATE_ID:
		//			fsi.writeDate((Date) object, writeInTransaction);
		//			break;
		//		case ODBType.STRING_ID:
		//			fsi.writeString((String) object, writeInTransaction, totalSpaceIfString);
		//			break;
		//		case ODBType.OBJECT_OID_ID:
		//			long oid = ((OdbObjectOID) object).getObjectId();
		//			fsi.writeLong(oid, writeInTransaction, "ODB OID", DefaultWriteAction.DATA_WRITE_ACTION);
		//			break;
		//		case ODBType.CLASS_OID_ID:
		//			oid = ((OdbClassOID) object).getObjectId();
		//			fsi.writeLong(oid, writeInTransaction, "ODB OID", DefaultWriteAction.DATA_WRITE_ACTION);
		//			break;
		//
		//		default:
		//			throw new RuntimeException("native type with odb type id " + odbTypeId + " (" + ODBType.getNameFromId(odbTypeId)
		//					+ ") for attribute ? is not suported");
		//		}
		//		return startPosition;
		//	}
		//
		//	/**
		//	 * Updates the previous object position field of the object at
		//	 * objectPosition
		//	 * 
		//	 * @param objectOID
		//	 * @param previousObjectOID
		//	 * @param writeInTransaction
		//	 * @throws IOException
		//	 */
		//	public void updatePreviousObjectFieldOfObjectInfo(OID objectOID, OID previousObjectOID, boolean writeInTransaction) throws IOException {
		//		long objectPosition = idManager.getObjectPositionWithOid(objectOID, true);
		//		fsi.setWritePosition(objectPosition + StorageEngineConstant.OBJECT_OFFSET_PREVIOUS_OBJECT_OID, writeInTransaction);
		//		writeOid(previousObjectOID, writeInTransaction, "prev object position", DefaultWriteAction.POINTER_WRITE_ACTION);
		//	}
		//
		//	/**
		//	 * Update next object oid field of the object at the specific position
		//	 * 
		//	 * @param objectOID
		//	 * @param nextObjectOID
		//	 * @param writeInTransaction
		//	 * @throws IOException
		//	 */
		//	public void updateNextObjectFieldOfObjectInfo(OID objectOID, OID nextObjectOID, boolean writeInTransaction) throws IOException {
		//		long objectPosition = idManager.getObjectPositionWithOid(objectOID, true);
		//		fsi.setWritePosition(objectPosition + StorageEngineConstant.OBJECT_OFFSET_NEXT_OBJECT_OID, writeInTransaction);
		//		writeOid(nextObjectOID, writeInTransaction, "next object oid of object info", DefaultWriteAction.POINTER_WRITE_ACTION);
		//	}
		//
		//	/**
		//	 * Mark a block as deleted
		//	 * 
		//	 * @return The block size
		//	 * 
		//	 * @param currentPosition
		//	 * @throws IOException
		//	 */
		//
		//	protected int markAsDeleted(long currentPosition, OID oid, boolean writeInTransaction) throws IOException {
		//
		//		fsi.setReadPosition(currentPosition);
		//		int blockSize = fsi.readInt();
		//		fsi.setWritePosition(currentPosition + StorageEngineConstant.NATIVE_OBJECT_OFFSET_BLOCK_TYPE, writeInTransaction);
		//		// Do not write block size, leave it as it is, to know the available
		//		// space for future use
		//		fsi.writeByte(BlockTypes.BLOCK_TYPE_DELETED, writeInTransaction);
		//
		//		storeFreeSpace(currentPosition, blockSize);
		//
		//		return blockSize;
		//	}
		//
		//	protected void storeFreeSpace(long currentPosition, int blockSize) {
		//		if (Configuration.isDebugEnabled(LOG_ID)) {
		//			DLogger.debug("Storing free space at position " + currentPosition + " | block size = " + blockSize);
		//		}
		//	}
		//
		//	/**
		//	 * Writes a pointer block : A pointer block is like a goto. It can be used
		//	 * for example when an instance has been updated. To enable all the
		//	 * references to it to be updated, we just create o pointer at the place of
		//	 * the updated instance. When searching for the instance, if the block type
		//	 * is POINTER, then the position will be set to the pointer position
		//	 * 
		//	 * @param currentPosition
		//	 * @param newObjectPosition
		//	 * @throws IOException
		//	 */
		//	protected void markAsAPointerTo(OID oid, long currentPosition, long newObjectPosition) throws IOException {
		//		throw new ODBRuntimeException(Error.FOUND_POINTER.addParameter(oid.getObjectId()).addParameter(newObjectPosition));
		//
		//		/*
		//		 * 
		//		 * if (currentPosition == newObjectPosition) { throw new
		//		 * ODBRuntimeException(Error.POINTER_TO_SELF.addParameter(currentPosition).addParameter(newObjectPosition).addParameter(
		//		 * oid)); } boolean writeInTransaction = true;
		//		 * fsi.setWritePosition(currentPosition, true);
		//		 * fsi.writeInt(ODBType.NATIVE_INT.getSize() +
		//		 * ODBType.NATIVE_BYTE.getSize() + ODBType.NATIVE_LONG.getSize(),
		//		 * writeInTransaction, "block size");
		//		 * fsi.writeByte(BlockTypes.BLOCK_TYPE_POINTER, writeInTransaction);
		//		 * fsi.writeLong(newObjectPosition, writeInTransaction, "pointer object
		//		 * position", WriteAction.POINTER_WRITE_ACTION); //new
		//		 * Exception().printStackTrace();
		//		 * 
		//		 */
		//	}
		//
		//	/**
		//	 * Updates the instance related field of the class info into the database
		//	 * file Updates the number of objects, the first object oid and the next
		//	 * class oid
		//	 * 
		//	 * @param classInfo
		//	 *            The class info to be updated
		//	 * @param writeInTransaction
		//	 *            To specify if it must be part of a transaction
		//	 * @throws IOException
		//	 */
		//	public void updateInstanceFieldsOfClassInfo(ClassInfo classInfo, boolean writeInTransaction) throws IOException {
		//		long currentPosition = fsi.getPosition();
		//		if (Configuration.isDebugEnabled(LOG_ID_DEBUG)) {
		//			DLogger.debug(depthToSpaces() + "Start of updateInstanceFieldsOfClassInfo for " + classInfo.getFullClassName());
		//		}
		//		long position = classInfo.getPosition() + StorageEngineConstant.CLASS_OFFSET_CLASS_NB_OBJECTS;
		//		fsi.setWritePosition(position, writeInTransaction);
		//
		//		long nbObjects = classInfo.getNumberOfObjects();
		//
		//		fsi.writeLong(nbObjects, writeInTransaction, "class info update nb objects", DefaultWriteAction.POINTER_WRITE_ACTION);
		//		writeOid(classInfo.getCommitedZoneInfo().first, writeInTransaction, "class info update first obj oid",
		//				DefaultWriteAction.POINTER_WRITE_ACTION);
		//		writeOid(classInfo.getCommitedZoneInfo().last, writeInTransaction, "class info update last obj oid",
		//				DefaultWriteAction.POINTER_WRITE_ACTION);
		//		if (Configuration.isDebugEnabled(LOG_ID_DEBUG)) {
		//			DLogger.debug(depthToSpaces() + "End of updateInstanceFieldsOfClassInfo for " + classInfo.getFullClassName());
		//		}
		//		fsi.setWritePosition(currentPosition, writeInTransaction);
		//	}
		//
		//	/**
		//	 * Updates the last instance field of the class info into the database file
		//	 * 
		//	 * @param classInfoPosition
		//	 *            The class info to be updated
		//	 * @param lastInstancePosition
		//	 *            The last instance position
		//	 * @throws IOException
		//	 */
		//	protected void updateLastInstanceFieldOfClassInfoWithId(OID classInfoId, long lastInstancePosition) throws IOException {
		//		long currentPosition = fsi.getPosition();
		//		// TODO CHECK LOGIC of getting position of class using this method for
		//		// object)
		//		long classInfoPosition = idManager.getObjectPositionWithOid(classInfoId, true);
		//
		//		fsi.setWritePosition(classInfoPosition + StorageEngineConstant.CLASS_OFFSET_CLASS_LAST_OBJECT_POSITION, true);
		//		fsi.writeLong(lastInstancePosition, true, "class info update last instance field", DefaultWriteAction.POINTER_WRITE_ACTION);
		//
		//		// TODO check if we need this
		//		fsi.setWritePosition(currentPosition, true);
		//	}
		//
		//	/**
		//	 * Updates the first instance field of the class info into the database file
		//	 * 
		//	 * @param classInfoPosition
		//	 *            The class info to be updated
		//	 * @param firstInstancePosition
		//	 *            The first instance position
		//	 * @throws IOException
		//	 */
		//	protected void updateFirstInstanceFieldOfClassInfoWithId(OID classInfoId, long firstInstancePosition) throws IOException {
		//		long currentPosition = fsi.getPosition();
		//
		//		// TODO CHECK LOGIC of getting position of class using this method for
		//		// object)
		//		long classInfoPosition = idManager.getObjectPositionWithOid(classInfoId, true);
		//
		//		fsi.setWritePosition(classInfoPosition + StorageEngineConstant.CLASS_OFFSET_CLASS_FIRST_OBJECT_POSITION, true);
		//		fsi.writeLong(firstInstancePosition, true, "class info update first instance field", DefaultWriteAction.POINTER_WRITE_ACTION);
		//
		//		// TODO check if we need this
		//		fsi.setWritePosition(currentPosition, true);
		//	}
		//
		//	/**
		//	 * Updates the number of objects of the class info into the database file
		//	 * 
		//	 * @param classInfoPosition
		//	 *            The class info to be updated
		//	 * @param nbObjects
		//	 *            The number of object
		//	 * @throws IOException
		//	 */
		//
		//	protected void updateNbObjectsFieldOfClassInfo(OID classInfoId, long nbObjects) throws IOException {
		//		long currentPosition = fsi.getPosition();
		//		long classInfoPosition = getSession().getMetaModel().getClassInfoFromId(classInfoId).getPosition();
		//		fsi.setWritePosition(classInfoPosition + StorageEngineConstant.CLASS_OFFSET_CLASS_NB_OBJECTS, true);
		//		fsi.writeLong(nbObjects, true, "class info update nb objects", DefaultWriteAction.POINTER_WRITE_ACTION);
		//
		//		// TODO check if we need this
		//		fsi.setWritePosition(currentPosition, true);
		//	}
		//
		//	/**
		//	 * <pre>
		//	 *                      Class User{
		//	 *                       private String name;
		//	 *                       private Function function;
		//	 *                     }
		//	 *                     
		//	 *                      When an object of type User is stored, it stores a reference to its function object.
		//	 *                      If the function is set to another, the pointer to the function object must be changed.
		//	 *                      for example, it was pointing to a function at the position 1407, the 1407 value is stored while
		//	 *                      writing the USer object, let's say at the position 528. To make the user point to another function object (which exist at the position 1890)
		//	 *                      The position 528 must be updated to 1890.
		//	 *                     
		//	 *                     
		//	 * </pre>
		//	 * 
		//	 * @param positionWhereTheReferenceIsStored
		//	 * @param newOid
		//	 * @throws IOException
		//	 */
		//	public void updateObjectReference(long positionWhereTheReferenceIsStored, OID newOid, boolean writeInTransaction) throws IOException {
		//		long position = positionWhereTheReferenceIsStored;
		//		if (position < 0) {
		//			throw new ODBRuntimeException(Error.NEGATIVE_POSITION.addParameter(position));
		//			/*
		//			 * long id = position; // This is an id position =
		//			 * objectReader.getObjectPositionFromItsOid(id, true);
		//			 */
		//		}
		//		fsi.setWritePosition(position, writeInTransaction);
		//		// Ids are always stored as negative value to differ from a position!
		//		long oid = StorageEngineConstant.NULL_OBJECT_ID_ID;
		//		if (newOid != null) {
		//			oid = -newOid.getObjectId();
		//		}
		//		fsi.writeLong(oid, writeInTransaction, "object reference", DefaultWriteAction.POINTER_WRITE_ACTION);
		//	}
		//
		//	/**
		//	 * In place update for array element, only do in place update for atomic
		//	 * native fixed size elements
		//	 * 
		//	 * @param arrayPosition
		//	 * @param arrayElementIndexToChange
		//	 * @param newValue
		//	 * @return true if in place update has been done,false if not
		//	 * @throws Exception
		//	 */
		//	private boolean updateArrayElement(long arrayPosition, int arrayElementIndexToChange, NativeObjectInfo newValue,
		//			boolean writeInTransaction) throws Exception {
		//		// block size, block type, odb typeid,is null?
		//		long offset = ODBType.INTEGER.getSize() + ODBType.BYTE.getSize() + ODBType.INTEGER.getSize() + ODBType.BOOLEAN.getSize();
		//		fsi.setReadPosition(arrayPosition + offset);
		//		// read class name of array elements
		//		String arrayElementClassName = fsi.readString();
		//		// TODO try to get array element type from the ArrayObjectInfo
		//		// Check if the class has fixed size : array support in place update
		//		// only for fixed size class like int, long, date,...
		//		// String array,for example do not support in place update
		//		ODBType arrayElementType = ODBType.getFromName(arrayElementClassName);
		//		if (!arrayElementType.isAtomicNative() || !arrayElementType.hasFixSize()) {
		//			return false;
		//		}
		//		ArrayObjectInfo a = null;
		//
		//		// reads the size of the array
		//		int arraySize = fsi.readInt();
		//
		//		if (arrayElementIndexToChange >= arraySize) {
		//			throw new ODBRuntimeException(Error.INPLACE_UPDATE_NOT_POSSIBLE_FOR_ARRAY.addParameter(arraySize).addParameter(
		//					arrayElementIndexToChange));
		//		}
		//
		//		// Gets the position where to write the object
		//		// Skip the positions where we have the pointers to each array element
		//		// then
		//		// jump to the right position
		//		long skip = arrayElementIndexToChange * ODBType.LONG.getSize();
		//
		//		fsi.setReadPosition(fsi.getPosition() + skip);
		//		long elementArrayPosition = fsi.readLong();
		//
		//		fsi.setWritePosition(elementArrayPosition, writeInTransaction);
		//		// Actually update the array element
		//		writeNativeObjectInfo(newValue, elementArrayPosition, true, writeInTransaction);
		//
		//		return true;
		//
		//	}
		//
		//	public void flush() throws IOException {
		//		fsi.flush();
		//	}
		//
		//	public IIdManager getIdManager() {
		//		return idManager;
		//	}
		//
		//	public ISession getSession() {
		//		return session;
		//	}
		//
		//	public void clear() {
		//		objectReader = null;
		//		if (idManager != null) {
		//			idManager.clear();
		//			idManager = null;
		//		}
		//
		//		storageEngine = null;
		//		session = null;
		//		fsi = null;
		//
		//	}
		//
		//	public static int getNbInPlaceUpdates() {
		//		return nbInPlaceUpdates;
		//	}
		//
		//	public static void setNbInPlaceUpdates(int nbInPlaceUpdates) {
		//		ObjectWriterNew.nbInPlaceUpdates = nbInPlaceUpdates;
		//	}
		//
		//	public static int getNbNormalUpdates() {
		//		return nbNormalUpdates;
		//	}
		//
		//	public static void setNbNormalUpdates(int nbNormalUpdates) {
		//		ObjectWriterNew.nbNormalUpdates = nbNormalUpdates;
		//	}
		//
		//	public static void resetNbUpdates() {
		//		nbInPlaceUpdates = 0;
		//		nbNormalUpdates = 0;
		//	}
	}
}
