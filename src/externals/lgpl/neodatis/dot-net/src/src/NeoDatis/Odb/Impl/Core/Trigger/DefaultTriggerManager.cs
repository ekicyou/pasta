namespace NeoDatis.Odb.Impl.Core.Trigger
{
	public class DefaultTriggerManager : NeoDatis.Odb.Core.Trigger.ITriggerManager
	{
		private static readonly string AllClassTrigger = "__all_class_";

		private NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine storageEngine;

		/// <summary>key is class Name, value is the collection of triggers for the class</summary>
		protected System.Collections.Generic.IDictionary<string, NeoDatis.Tool.Wrappers.List.IOdbList
			<NeoDatis.Odb.Core.Trigger.Trigger>> listOfUpdateTriggers;

		/// <summary>key is class Name, value is the collection of triggers for the class</summary>
		protected System.Collections.Generic.IDictionary<string, NeoDatis.Tool.Wrappers.List.IOdbList
			<NeoDatis.Odb.Core.Trigger.Trigger>> listOfInsertTriggers;

		/// <summary>key is class Name, value is the collection of triggers for the class</summary>
		protected System.Collections.Generic.IDictionary<string, NeoDatis.Tool.Wrappers.List.IOdbList
			<NeoDatis.Odb.Core.Trigger.Trigger>> listOfDeleteTriggers;

		/// <summary>key is class Name, value is the collection of triggers for the class</summary>
		protected System.Collections.Generic.IDictionary<string, NeoDatis.Tool.Wrappers.List.IOdbList
			<NeoDatis.Odb.Core.Trigger.Trigger>> listOfSelectTriggers;

		public DefaultTriggerManager(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine
			)
		{
			this.storageEngine = engine;
			Init();
		}

		private void Init()
		{
			listOfUpdateTriggers = new NeoDatis.Tool.Wrappers.Map.OdbHashMap<string, NeoDatis.Tool.Wrappers.List.IOdbList
				<NeoDatis.Odb.Core.Trigger.Trigger>>();
			listOfDeleteTriggers = new NeoDatis.Tool.Wrappers.Map.OdbHashMap<string, NeoDatis.Tool.Wrappers.List.IOdbList
				<NeoDatis.Odb.Core.Trigger.Trigger>>();
			listOfSelectTriggers = new NeoDatis.Tool.Wrappers.Map.OdbHashMap<string, NeoDatis.Tool.Wrappers.List.IOdbList
				<NeoDatis.Odb.Core.Trigger.Trigger>>();
			listOfInsertTriggers = new NeoDatis.Tool.Wrappers.Map.OdbHashMap<string, NeoDatis.Tool.Wrappers.List.IOdbList
				<NeoDatis.Odb.Core.Trigger.Trigger>>();
		}

		public virtual void AddUpdateTriggerFor(string className, NeoDatis.Odb.Core.Trigger.UpdateTrigger
			 trigger)
		{
			if (className == null)
			{
				className = AllClassTrigger;
			}
			NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Trigger.Trigger> c = listOfUpdateTriggers
				[className];
			if (c == null)
			{
				c = new NeoDatis.Tool.Wrappers.List.OdbArrayList<NeoDatis.Odb.Core.Trigger.Trigger
					>();
				listOfUpdateTriggers.Add(className, c);
			}
			c.Add(trigger);
		}

		public virtual void AddInsertTriggerFor(string className, NeoDatis.Odb.Core.Trigger.InsertTrigger
			 trigger)
		{
			if (className == null)
			{
				className = AllClassTrigger;
			}
			NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Trigger.Trigger> c = listOfInsertTriggers
				[className];
			if (c == null)
			{
				c = new NeoDatis.Tool.Wrappers.List.OdbArrayList<NeoDatis.Odb.Core.Trigger.Trigger
					>();
				listOfInsertTriggers.Add(className, c);
			}
			c.Add(trigger);
		}

		public virtual void AddDeleteTriggerFor(string className, NeoDatis.Odb.Core.Trigger.DeleteTrigger
			 trigger)
		{
			if (className == null)
			{
				className = AllClassTrigger;
			}
			NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Trigger.Trigger> c = listOfDeleteTriggers
				[className];
			if (c == null)
			{
				c = new NeoDatis.Tool.Wrappers.List.OdbArrayList<NeoDatis.Odb.Core.Trigger.Trigger
					>();
				listOfDeleteTriggers.Add(className, c);
			}
			c.Add(trigger);
		}

		public virtual void AddSelectTriggerFor(string className, NeoDatis.Odb.Core.Trigger.SelectTrigger
			 trigger)
		{
			if (className == null)
			{
				className = AllClassTrigger;
			}
			NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Trigger.Trigger> c = listOfSelectTriggers
				[className];
			if (c == null)
			{
				c = new NeoDatis.Tool.Wrappers.List.OdbArrayList<NeoDatis.Odb.Core.Trigger.Trigger
					>();
				listOfSelectTriggers.Add(className, c);
			}
			c.Add(trigger);
		}

		public virtual bool HasDeleteTriggersFor(string classsName)
		{
			return listOfDeleteTriggers.ContainsKey(classsName) || listOfDeleteTriggers.ContainsKey
				(AllClassTrigger);
		}

		public virtual bool HasInsertTriggersFor(string className)
		{
			return listOfInsertTriggers.ContainsKey(className) || listOfInsertTriggers.ContainsKey
				(AllClassTrigger);
		}

		public virtual bool HasSelectTriggersFor(string className)
		{
			return listOfSelectTriggers.ContainsKey(className) || listOfSelectTriggers.ContainsKey
				(AllClassTrigger);
		}

		public virtual bool HasUpdateTriggersFor(string className)
		{
			return listOfUpdateTriggers.ContainsKey(className) || listOfUpdateTriggers.ContainsKey
				(AllClassTrigger);
		}

		/// <summary>FIXME try to cache l1+l2</summary>
		/// <param name="className"></param>
		/// <returns></returns>
		public virtual NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Trigger.Trigger
			> GetListOfDeleteTriggersFor(string className)
		{
			NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Trigger.Trigger> l1 = listOfDeleteTriggers
				[className];
			NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Trigger.Trigger> l2 = listOfDeleteTriggers
				[AllClassTrigger];
			if (l2 != null)
			{
				int size = l2.Count;
				if (l1 != null)
				{
					size = size + l1.Count;
				}
				NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Trigger.Trigger> r = new NeoDatis.Tool.Wrappers.List.OdbArrayList
					<NeoDatis.Odb.Core.Trigger.Trigger>(size);
				if (l1 != null)
				{
					r.AddAll(l1);
				}
				r.AddAll(l2);
				return r;
			}
			return l1;
		}

		public virtual NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Trigger.Trigger
			> GetListOfInsertTriggersFor(string className)
		{
			NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Trigger.Trigger> l1 = listOfInsertTriggers
				[className];
			NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Trigger.Trigger> l2 = listOfInsertTriggers
				[AllClassTrigger];
			if (l2 != null)
			{
				int size = l2.Count;
				if (l1 != null)
				{
					size = size + l1.Count;
				}
				NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Trigger.Trigger> r = new NeoDatis.Tool.Wrappers.List.OdbArrayList
					<NeoDatis.Odb.Core.Trigger.Trigger>(size);
				if (l1 != null)
				{
					r.AddAll(l1);
				}
				r.AddAll(l2);
				return r;
			}
			return l1;
		}

		public virtual NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Trigger.Trigger
			> GetListOfSelectTriggersFor(string className)
		{
			NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Trigger.Trigger> l1 = listOfSelectTriggers
				[className];
			NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Trigger.Trigger> l2 = listOfSelectTriggers
				[AllClassTrigger];
			if (l2 != null)
			{
				int size = l2.Count;
				if (l1 != null)
				{
					size = size + l1.Count;
				}
				NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Trigger.Trigger> r = new NeoDatis.Tool.Wrappers.List.OdbArrayList
					<NeoDatis.Odb.Core.Trigger.Trigger>(size);
				if (l1 != null)
				{
					r.AddAll(l1);
				}
				r.AddAll(l2);
				return r;
			}
			return l1;
		}

		public virtual NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Trigger.Trigger
			> GetListOfUpdateTriggersFor(string className)
		{
			NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Trigger.Trigger> l1 = listOfUpdateTriggers
				[className];
			NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Trigger.Trigger> l2 = listOfUpdateTriggers
				[AllClassTrigger];
			if (l2 != null)
			{
				int size = l2.Count;
				if (l1 != null)
				{
					size = size + l1.Count;
				}
				NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Trigger.Trigger> r = new NeoDatis.Tool.Wrappers.List.OdbArrayList
					<NeoDatis.Odb.Core.Trigger.Trigger>(size);
				if (l1 != null)
				{
					r.AddAll(l1);
				}
				r.AddAll(l2);
				return r;
			}
			return l1;
		}

		public virtual bool ManageInsertTriggerBefore(string className, object @object)
		{
			if (HasInsertTriggersFor(className))
			{
				NeoDatis.Odb.Core.Trigger.InsertTrigger trigger = null;
				System.Collections.IEnumerator iterator = GetListOfInsertTriggersFor(className).GetEnumerator
					();
				while (iterator.MoveNext())
				{
					trigger = (NeoDatis.Odb.Core.Trigger.InsertTrigger)iterator.Current;
					if (trigger.GetOdb() == null)
					{
						trigger.SetOdb(new NeoDatis.Odb.Impl.Main.ODBForTrigger(storageEngine));
					}
					try
					{
						if (!IsNull(@object))
						{
							trigger.BeforeInsert(Transform(@object));
						}
					}
					catch (System.Exception e)
					{
						NeoDatis.Odb.Core.IError warning = NeoDatis.Odb.Core.NeoDatisError.BeforeInsertTriggerHasThrownException
							.AddParameter(trigger.GetType().FullName).AddParameter(NeoDatis.Tool.Wrappers.OdbString
							.ExceptionToString(e, false));
						if (NeoDatis.Odb.OdbConfiguration.DisplayWarnings())
						{
							NeoDatis.Tool.DLogger.Info(warning);
						}
					}
				}
			}
			return true;
		}

		public virtual void ManageInsertTriggerAfter(string className, object @object, NeoDatis.Odb.OID
			 oid)
		{
			if (HasInsertTriggersFor(className))
			{
				NeoDatis.Odb.Core.Trigger.InsertTrigger trigger = null;
				System.Collections.IEnumerator iterator = GetListOfInsertTriggersFor(className).GetEnumerator
					();
				while (iterator.MoveNext())
				{
					trigger = (NeoDatis.Odb.Core.Trigger.InsertTrigger)iterator.Current;
					if (trigger.GetOdb() == null)
					{
						trigger.SetOdb(new NeoDatis.Odb.Impl.Main.ODBForTrigger(storageEngine));
					}
					try
					{
						trigger.AfterInsert(Transform(@object), oid);
					}
					catch (System.Exception e)
					{
						NeoDatis.Odb.Core.IError warning = NeoDatis.Odb.Core.NeoDatisError.AfterInsertTriggerHasThrownException
							.AddParameter(trigger.GetType().FullName).AddParameter(NeoDatis.Tool.Wrappers.OdbString
							.ExceptionToString(e, false));
						if (NeoDatis.Odb.OdbConfiguration.DisplayWarnings())
						{
							NeoDatis.Tool.DLogger.Info(warning);
						}
					}
				}
			}
		}

		public virtual bool ManageUpdateTriggerBefore(string className, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 oldNnoi, object newObject, NeoDatis.Odb.OID oid)
		{
			if (HasUpdateTriggersFor(className))
			{
				NeoDatis.Odb.Core.Trigger.UpdateTrigger trigger = null;
				System.Collections.IEnumerator iterator = GetListOfUpdateTriggersFor(className).GetEnumerator
					();
				while (iterator.MoveNext())
				{
					trigger = (NeoDatis.Odb.Core.Trigger.UpdateTrigger)iterator.Current;
					if (trigger.GetOdb() == null)
					{
						trigger.SetOdb(new NeoDatis.Odb.Impl.Main.ODBForTrigger(storageEngine));
					}
					try
					{
						trigger.BeforeUpdate(new NeoDatis.Odb.Impl.Core.Server.Trigger.DefaultObjectRepresentation
							(oldNnoi), Transform(newObject), oid);
					}
					catch (System.Exception e)
					{
						NeoDatis.Odb.Core.IError warning = NeoDatis.Odb.Core.NeoDatisError.BeforeUpdateTriggerHasThrownException
							.AddParameter(trigger.GetType().FullName).AddParameter(NeoDatis.Tool.Wrappers.OdbString
							.ExceptionToString(e, false));
						if (NeoDatis.Odb.OdbConfiguration.DisplayWarnings())
						{
							NeoDatis.Tool.DLogger.Info(warning);
						}
					}
				}
			}
			return true;
		}

		public virtual void ManageUpdateTriggerAfter(string className, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 oldNnoi, object newObject, NeoDatis.Odb.OID oid)
		{
			if (HasUpdateTriggersFor(className))
			{
				NeoDatis.Odb.Core.Trigger.UpdateTrigger trigger = null;
				System.Collections.IEnumerator iterator = GetListOfUpdateTriggersFor(className).GetEnumerator
					();
				while (iterator.MoveNext())
				{
					trigger = (NeoDatis.Odb.Core.Trigger.UpdateTrigger)iterator.Current;
					if (trigger.GetOdb() == null)
					{
						trigger.SetOdb(new NeoDatis.Odb.Impl.Main.ODBForTrigger(storageEngine));
					}
					try
					{
						trigger.AfterUpdate(new NeoDatis.Odb.Impl.Core.Server.Trigger.DefaultObjectRepresentation
							(oldNnoi), Transform(newObject), oid);
					}
					catch (System.Exception e)
					{
						NeoDatis.Odb.Core.IError warning = NeoDatis.Odb.Core.NeoDatisError.AfterUpdateTriggerHasThrownException
							.AddParameter(trigger.GetType().FullName).AddParameter(NeoDatis.Tool.Wrappers.OdbString
							.ExceptionToString(e, false));
						if (NeoDatis.Odb.OdbConfiguration.DisplayWarnings())
						{
							NeoDatis.Tool.DLogger.Info(warning);
						}
					}
				}
			}
		}

		public virtual bool ManageDeleteTriggerBefore(string className, object @object, NeoDatis.Odb.OID
			 oid)
		{
			if (HasDeleteTriggersFor(className))
			{
				NeoDatis.Odb.Core.Trigger.DeleteTrigger trigger = null;
				System.Collections.IEnumerator iterator = GetListOfDeleteTriggersFor(className).GetEnumerator
					();
				while (iterator.MoveNext())
				{
					trigger = (NeoDatis.Odb.Core.Trigger.DeleteTrigger)iterator.Current;
					if (trigger.GetOdb() == null)
					{
						trigger.SetOdb(new NeoDatis.Odb.Impl.Main.ODBForTrigger(storageEngine));
					}
					try
					{
						trigger.BeforeDelete(Transform(@object), oid);
					}
					catch (System.Exception e)
					{
						NeoDatis.Odb.Core.IError warning = NeoDatis.Odb.Core.NeoDatisError.BeforeDeleteTriggerHasThrownException
							.AddParameter(trigger.GetType().FullName).AddParameter(NeoDatis.Tool.Wrappers.OdbString
							.ExceptionToString(e, true));
						if (NeoDatis.Odb.OdbConfiguration.DisplayWarnings())
						{
							NeoDatis.Tool.DLogger.Info(warning);
						}
					}
				}
			}
			return true;
		}

		public virtual void ManageDeleteTriggerAfter(string className, object @object, NeoDatis.Odb.OID
			 oid)
		{
			if (HasDeleteTriggersFor(className))
			{
				NeoDatis.Odb.Core.Trigger.DeleteTrigger trigger = null;
				System.Collections.IEnumerator iterator = GetListOfDeleteTriggersFor(className).GetEnumerator
					();
				while (iterator.MoveNext())
				{
					trigger = (NeoDatis.Odb.Core.Trigger.DeleteTrigger)iterator.Current;
					if (trigger.GetOdb() == null)
					{
						trigger.SetOdb(new NeoDatis.Odb.Impl.Main.ODBForTrigger(storageEngine));
					}
					try
					{
						trigger.AfterDelete(Transform(@object), oid);
					}
					catch (System.Exception e)
					{
						NeoDatis.Odb.Core.IError warning = NeoDatis.Odb.Core.NeoDatisError.AfterDeleteTriggerHasThrownException
							.AddParameter(trigger.GetType().FullName).AddParameter(NeoDatis.Tool.Wrappers.OdbString
							.ExceptionToString(e, false));
						if (NeoDatis.Odb.OdbConfiguration.DisplayWarnings())
						{
							NeoDatis.Tool.DLogger.Info(warning);
						}
					}
				}
			}
		}

		public virtual void ManageSelectTriggerAfter(string className, object @object, NeoDatis.Odb.OID
			 oid)
		{
			if (HasSelectTriggersFor(className))
			{
				NeoDatis.Odb.Core.Trigger.SelectTrigger trigger = null;
				System.Collections.IEnumerator iterator = GetListOfSelectTriggersFor(className).GetEnumerator
					();
				while (iterator.MoveNext())
				{
					trigger = (NeoDatis.Odb.Core.Trigger.SelectTrigger)iterator.Current;
					if (trigger.GetOdb() == null)
					{
						trigger.SetOdb(new NeoDatis.Odb.Impl.Main.ODBForTrigger(storageEngine));
					}
					if (!IsNull(@object))
					{
						trigger.AfterSelect(Transform(@object), oid);
					}
				}
			}
		}

		protected virtual bool IsNull(object @object)
		{
			return @object == null;
		}

		/// <summary>For the default object trigger, no transformation is needed</summary>
		public virtual object Transform(object @object)
		{
			return @object;
		}
	}
}
