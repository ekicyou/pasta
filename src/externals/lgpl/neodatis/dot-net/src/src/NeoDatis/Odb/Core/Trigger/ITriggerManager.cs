namespace NeoDatis.Odb.Core.Trigger
{
	public interface ITriggerManager
	{
		bool ManageInsertTriggerBefore(string className, object @object);

		void ManageInsertTriggerAfter(string className, object @object, NeoDatis.Odb.OID 
			oid);

		bool ManageUpdateTriggerBefore(string className, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 oldObjectRepresentation, object newObject, NeoDatis.Odb.OID oid);

		void ManageUpdateTriggerAfter(string className, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 oldObjectRepresentation, object newObject, NeoDatis.Odb.OID oid);

		bool ManageDeleteTriggerBefore(string className, object @object, NeoDatis.Odb.OID
			 oid);

		void ManageDeleteTriggerAfter(string className, object @object, NeoDatis.Odb.OID 
			oid);

		void ManageSelectTriggerAfter(string className, object @object, NeoDatis.Odb.OID 
			oid);

		void AddUpdateTriggerFor(string className, NeoDatis.Odb.Core.Trigger.UpdateTrigger
			 trigger);

		void AddInsertTriggerFor(string className, NeoDatis.Odb.Core.Trigger.InsertTrigger
			 trigger);

		void AddDeleteTriggerFor(string className, NeoDatis.Odb.Core.Trigger.DeleteTrigger
			 trigger);

		void AddSelectTriggerFor(string className, NeoDatis.Odb.Core.Trigger.SelectTrigger
			 trigger);

		/// <summary>used to transform object before real trigger call.</summary>
		/// <remarks>
		/// used to transform object before real trigger call. This is used for
		/// example, in server side trigger where the object is encapsulated in an
		/// ObjectRepresentation instance. It is only for internal use
		/// </remarks>
		object Transform(object @object);

		bool HasDeleteTriggersFor(string classsName);

		bool HasInsertTriggersFor(string className);

		bool HasSelectTriggersFor(string className);

		bool HasUpdateTriggersFor(string className);
	}
}
