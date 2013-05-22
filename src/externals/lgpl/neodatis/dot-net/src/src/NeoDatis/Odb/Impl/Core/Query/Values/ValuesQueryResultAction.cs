namespace NeoDatis.Odb.Impl.Core.Query.Values
{
	public class ValuesQueryResultAction : NeoDatis.Odb.Core.Query.Execution.IMatchingObjectAction
	{
		private NeoDatis.Odb.Core.Query.IValuesQuery query;

		/// <summary>A copy of the query object actions</summary>
		private NeoDatis.Odb.Core.Query.Execution.IQueryFieldAction[] queryFieldActions;

		private long nbObjects;

		private NeoDatis.Odb.Values result;

		private bool queryHasOrderBy;

		/// <summary>An object to build instances</summary>
		protected NeoDatis.Odb.Core.Layers.Layer2.Instance.IInstanceBuilder instanceBuilder;

		protected NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo classInfo;

		private int returnArraySize;

		private NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine;

		public ValuesQueryResultAction(NeoDatis.Odb.Core.Query.IValuesQuery query, NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine
			 storageEngine, NeoDatis.Odb.Core.Layers.Layer2.Instance.IInstanceBuilder instanceBuilder
			) : base()
		{
			this.engine = storageEngine;
			this.query = query;
			this.queryHasOrderBy = query.HasOrderBy();
			this.instanceBuilder = instanceBuilder;
			this.returnArraySize = query.GetObjectActions().Count;
			System.Collections.IEnumerator iterator = query.GetObjectActions().GetEnumerator(
				);
			NeoDatis.Odb.Core.Query.Execution.IQueryFieldAction qfa = null;
			queryFieldActions = new NeoDatis.Odb.Core.Query.Execution.IQueryFieldAction[returnArraySize
				];
			int i = 0;
			while (iterator.MoveNext())
			{
				qfa = (NeoDatis.Odb.Core.Query.Execution.IQueryFieldAction)iterator.Current;
				queryFieldActions[i] = qfa.Copy();
				queryFieldActions[i].SetReturnInstance(query.ReturnInstance());
				queryFieldActions[i].SetInstanceBuilder(instanceBuilder);
				i++;
			}
		}

		public virtual void ObjectMatch(NeoDatis.Odb.OID oid, NeoDatis.Tool.Wrappers.OdbComparable
			 orderByKey)
		{
		}

		// This method os not used in Values Query API
		public virtual void ObjectMatch(NeoDatis.Odb.OID oid, object @object, NeoDatis.Tool.Wrappers.OdbComparable
			 orderByKey)
		{
			if (query.IsMultiRow())
			{
				NeoDatis.Odb.ObjectValues values = ConvertObject((NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap
					)@object);
				if (queryHasOrderBy)
				{
					result.AddWithKey(orderByKey, values);
				}
				else
				{
					result.Add(values);
				}
			}
			else
			{
				Compute((NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap)@object);
			}
		}

		private void Compute(NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap values
			)
		{
			for (int i = 0; i < returnArraySize; i++)
			{
				queryFieldActions[i].Execute(values.GetObjectInfoHeader().GetOid(), values);
			}
		}

		private NeoDatis.Odb.ObjectValues ConvertObject(NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap
			 values)
		{
			NeoDatis.Odb.Impl.Core.Query.List.Values.DefaultObjectValues dov = new NeoDatis.Odb.Impl.Core.Query.List.Values.DefaultObjectValues
				(returnArraySize);
			NeoDatis.Odb.Core.Query.Execution.IQueryFieldAction qfa = null;
			for (int i = 0; i < returnArraySize; i++)
			{
				qfa = queryFieldActions[i];
				qfa.Execute(values.GetObjectInfoHeader().GetOid(), values);
				object o = qfa.GetValue();
				// When Values queries return objects, they actually return the oid of the object
				// So we must load it here
				if (o != null && o is NeoDatis.Odb.OID)
				{
					NeoDatis.Odb.Impl.Core.Oid.OdbObjectOID oid = (NeoDatis.Odb.Impl.Core.Oid.OdbObjectOID
						)o;
					o = engine.GetObjectFromOid(oid);
				}
				dov.Set(i, qfa.GetAlias(), o);
			}
			return dov;
		}

		public virtual void Start()
		{
			if (query != null && query.HasOrderBy())
			{
				result = new NeoDatis.Odb.Impl.Core.Query.List.Values.InMemoryBTreeCollectionForValues
					((int)nbObjects, query.GetOrderByType());
			}
			else
			{
				result = new NeoDatis.Odb.Impl.Core.Query.List.Values.SimpleListForValues((int)nbObjects
					);
			}
			NeoDatis.Odb.Core.Query.Execution.IQueryFieldAction qfa = null;
			for (int i = 0; i < returnArraySize; i++)
			{
				qfa = queryFieldActions[i];
				qfa.Start();
			}
		}

		public virtual void End()
		{
			NeoDatis.Odb.Core.Query.Execution.IQueryFieldAction qfa = null;
			NeoDatis.Odb.Impl.Core.Query.List.Values.DefaultObjectValues dov = null;
			if (!query.IsMultiRow())
			{
				dov = new NeoDatis.Odb.Impl.Core.Query.List.Values.DefaultObjectValues(returnArraySize
					);
			}
			for (int i = 0; i < returnArraySize; i++)
			{
				qfa = queryFieldActions[i];
				qfa.End();
				if (!query.IsMultiRow())
				{
					object o = qfa.GetValue();
					// When Values queries return objects, they actually return the oid of the object
					// So we must load it here
					if (o != null && o is NeoDatis.Odb.OID)
					{
						NeoDatis.Odb.Impl.Core.Oid.OdbObjectOID oid = (NeoDatis.Odb.Impl.Core.Oid.OdbObjectOID
							)o;
						o = engine.GetObjectFromOid(oid);
					}
					// Sets the values now
					dov.Set(i, qfa.GetAlias(), o);
				}
			}
			if (!query.IsMultiRow())
			{
				result.Add(dov);
			}
		}

		public virtual NeoDatis.Odb.Values GetValues()
		{
			return result;
		}

		public virtual NeoDatis.Odb.Objects<T> GetObjects<T>()
		{
			return (NeoDatis.Odb.Objects<T>)result;
		}
	}
}
