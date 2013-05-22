namespace NeoDatis.Odb.Core.Layers.Layer3
{
	public interface IObjectWriter : NeoDatis.Odb.Core.ITwoPhaseInit
	{
		NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoList AddClasses(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoList
			 classInfoList);

		/// <summary>Write the class info header to the database file</summary>
		/// <param name="classInfo">The class info to be written</param>
		/// <param name="position">The position at which it must be written</param>
		/// <param name="writeInTransaction">
		/// true if the write must be done in transaction, false to write
		/// directly
		/// </param>
		void WriteClassInfoHeader(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo classInfo
			, long position, bool writeInTransaction);

		void UpdateClassInfo(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo classInfo, bool
			 writeInTransaction);

		/// <summary>Updates an object.</summary>
		/// <remarks>
		/// Updates an object.
		/// <pre>
		/// Try to update in place. Only change what has changed. This is restricted to particular types (fixed size types). If in place update is
		/// not possible, then deletes the current object and creates a new at the end of the database file and updates
		/// OID object position.
		/// </remarks>
		/// <param name="nnoi">The meta representation of the object to be updated</param>
		/// <param name="forceUpdate">when true, no verification is done to check if update must be done.
		/// 	</param>
		/// <returns>The oid of the object, as a negative number</returns>
		NeoDatis.Odb.OID UpdateNonNativeObjectInfo(NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 nnoi, bool forceUpdate);

		/// <summary>Write an object representation to database file</summary>
		/// <param name="existingOid">The oid of the object, can be null</param>
		/// <param name="objectInfo">The Object meta representation</param>
		/// <param name="position">The position where the object must be written, can be -1</param>
		/// <param name="writeDataInTransaction">To indicate if the write must be done in or out of transaction
		/// 	</param>
		/// <returns>The oid of the object</returns>
		/// <exception cref="System.Exception">System.Exception</exception>
		NeoDatis.Odb.OID WriteNonNativeObjectInfo(NeoDatis.Odb.OID existingOid, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 objectInfo, long position, bool writeDataInTransaction, bool isNewObject);

		long WriteAtomicNativeObject(NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo
			 anoi, bool writeInTransaction, int totalSpaceIfString);

		NeoDatis.Odb.Core.Layers.Layer3.IIdManager GetIdManager();

		NeoDatis.Odb.Core.Transaction.ISession GetSession();

		void Close();

		NeoDatis.Odb.Core.Layers.Layer3.Engine.IFileSystemInterface GetFsi();

		/// <summary>Creates the header of the file</summary>
		/// <param name="creationDate">The creation date</param>
		/// <param name="user">The user</param>
		/// <param name="password">The password</param>
		void CreateEmptyDatabaseHeader(long creationDate, string user, string password);

		/// <summary>Mark a block as deleted</summary>
		/// <returns>The block size</returns>
		/// <param name="currentPosition"></param>
		int MarkAsDeleted(long currentPosition, NeoDatis.Odb.OID oid, bool writeInTransaction
			);

		/// <summary>Insert the object in the index</summary>
		/// <param name="oid">The object id</param>
		/// <param name="nnoi">The object meta represenation</param>
		/// <returns>The number of indexes</returns>
		int ManageIndexesForInsert(NeoDatis.Odb.OID oid, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 nnoi);

		/// <summary>Insert the object in the index</summary>
		/// <param name="oid">The object id</param>
		/// <param name="nnoi">The object meta represenation</param>
		/// <returns>The number of indexes</returns>
		/// <exception cref="System.Exception">System.Exception</exception>
		int ManageIndexesForDelete(NeoDatis.Odb.OID oid, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 nnoi);

		int ManageIndexesForUpdate(NeoDatis.Odb.OID oid, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 nnoi, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo oldMetaRepresentation
			);

		/// <summary>Write the status of the last odb close</summary>
		void WriteLastODBCloseStatus(bool ok, bool writeInTransaction);

		void Flush();

		NeoDatis.Odb.OID Delete(NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader header
			);

		void UpdateStatusForIdWithPosition(long idPosition, byte newStatus, bool writeInTransaction
			);

		/// <summary>Updates the real object position of the object OID</summary>
		/// <param name="idPosition">The OID position</param>
		/// <param name="objectPosition">The real object position</param>
		/// <param name="writeInTransaction">To indicate if write must be done in transaction
		/// 	</param>
		void UpdateObjectPositionForObjectOIDWithPosition(long idPosition, long objectPosition
			, bool writeInTransaction);

		/// <summary>Udates the real class positon of the class OID</summary>
		/// <param name="idPosition"></param>
		/// <param name="objectPosition"></param>
		/// <param name="writeInTransaction"></param>
		void UpdateClassPositionForClassOIDWithPosition(long idPosition, long objectPosition
			, bool writeInTransaction);

		/// <summary>Associate an object OID to its position</summary>
		/// <param name="idType">The type : can be object or class</param>
		/// <param name="idStatus">The status of the OID</param>
		/// <param name="currentBlockIdPosition">The current OID block position</param>
		/// <param name="oid">The OID</param>
		/// <param name="objectPosition">The position</param>
		/// <param name="writeInTransaction">To indicate if write must be executed in transaction
		/// 	</param>
		/// <returns></returns>
		long AssociateIdToObject(byte idType, byte idStatus, long currentBlockIdPosition, 
			NeoDatis.Odb.OID oid, long objectPosition, bool writeInTransaction);

		/// <summary>
		/// Marks a block of type id as full, changes the status and the next block
		/// position
		/// </summary>
		/// <param name="blockPosition"></param>
		/// <param name="nextBlockPosition"></param>
		/// <param name="writeInTransaction"></param>
		/// <returns>The block position</returns>
		long MarkIdBlockAsFull(long blockPosition, long nextBlockPosition, bool writeInTransaction
			);

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
		/// <returns>The position of the id</returns>
		long WriteIdBlock(long position, int idBlockSize, byte blockStatus, int blockNumber
			, long previousBlockPosition, bool writeInTransaction);

		/// <summary>
		/// Updates the previous object position field of the object at
		/// objectPosition
		/// </summary>
		/// <param name="objectOID"></param>
		/// <param name="previousObjectOID"></param>
		/// <param name="writeInTransaction"></param>
		void UpdatePreviousObjectFieldOfObjectInfo(NeoDatis.Odb.OID objectOID, NeoDatis.Odb.OID
			 previousObjectOID, bool writeInTransaction);

		/// <summary>Update next object oid field of the object at the specific position</summary>
		/// <param name="objectOID"></param>
		/// <param name="nextObjectOID"></param>
		/// <param name="writeInTransaction"></param>
		void UpdateNextObjectFieldOfObjectInfo(NeoDatis.Odb.OID objectOID, NeoDatis.Odb.OID
			 nextObjectOID, bool writeInTransaction);

		/// <summary>
		/// Updates the instance related field of the class info into the database
		/// file Updates the number of objects, the first object oid and the next
		/// class oid
		/// </summary>
		/// <param name="classInfo">The class info to be updated</param>
		/// <param name="writeInTransaction">To specify if it must be part of a transaction</param>
		void UpdateInstanceFieldsOfClassInfo(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			 classInfo, bool writeInTransaction);

		void AfterInit();

		NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo AddClass(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			 newClassInfo, bool addDependentClasses);

		/// <summary>Persist a single class info - This method is used by the XML Importer.</summary>
		/// <remarks>Persist a single class info - This method is used by the XML Importer.</remarks>
		NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo PersistClass(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			 newClassInfo, int lastClassInfoIndex, bool addClass, bool addDependentClasses);

		void WriteLastTransactionId(NeoDatis.Odb.TransactionId transactionId);

		void SetTriggerManager(NeoDatis.Odb.Core.Trigger.ITriggerManager triggerManager);

		/// <param name="oid">The Oid of the object to be inserted</param>
		/// <param name="nnoi">
		/// The object meta representation The object to be inserted in
		/// the database
		/// </param>
		/// <param name="isNewObject">To indicate if object is new</param>
		/// <returns>The position of the inserted object</returns>
		NeoDatis.Odb.OID InsertNonNativeObject(NeoDatis.Odb.OID oid, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 nnoi, bool isNewObject);
	}
}
