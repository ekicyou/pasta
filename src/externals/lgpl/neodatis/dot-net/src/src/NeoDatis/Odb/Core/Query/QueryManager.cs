namespace NeoDatis.Odb.Core.Query
{
	public class QueryManager
	{
		public static bool Match(NeoDatis.Odb.Core.Query.IQuery query, object @object)
		{
			if (typeof(NeoDatis.Odb.Core.Query.NQ.NativeQuery).IsAssignableFrom(query.GetType
				()))
			{
				return NeoDatis.Odb.Impl.Core.Query.NQ.NativeQueryManager.Match((NeoDatis.Odb.Core.Query.NQ.NativeQuery
					)query, @object);
			}
			if (typeof(NeoDatis.Odb.Core.Query.NQ.SimpleNativeQuery).IsAssignableFrom(query.GetType
				()))
			{
				return NeoDatis.Odb.Impl.Core.Query.NQ.NativeQueryManager.Match((NeoDatis.Odb.Core.Query.NQ.SimpleNativeQuery
					)query, @object);
			}
			if (typeof(NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery).IsAssignableFrom(
				query.GetType()))
			{
				return NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQueryManager.Match((NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
					)query, @object);
			}
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.QueryTypeNotImplemented
				.AddParameter(query.GetType().FullName));
		}

		public static string GetFullClassName(NeoDatis.Odb.Core.Query.IQuery query)
		{
			if (typeof(NeoDatis.Odb.Core.Query.NQ.NativeQuery).IsAssignableFrom(query.GetType
				()))
			{
				return NeoDatis.Odb.Impl.Core.Query.NQ.NativeQueryManager.GetClass((NeoDatis.Odb.Core.Query.NQ.NativeQuery
					)query);
			}
			if (typeof(NeoDatis.Odb.Core.Query.NQ.SimpleNativeQuery).IsAssignableFrom(query.GetType
				()))
			{
				return NeoDatis.Odb.Impl.Core.Query.NQ.NativeQueryManager.GetFullClassName((NeoDatis.Odb.Core.Query.NQ.SimpleNativeQuery
					)query);
			}
			if (typeof(NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery) == query.GetType(
				) || typeof(NeoDatis.Odb.Impl.Core.Query.Values.ValuesCriteriaQuery) == query.GetType
				())
			{
				return NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQueryManager.GetFullClassName
					((NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery)query);
			}
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.QueryTypeNotImplemented
				.AddParameter(query.GetType().FullName));
		}

		public static bool NeedsInstanciation(NeoDatis.Odb.Core.Query.IQuery query)
		{
			if (typeof(NeoDatis.Odb.Core.Query.NQ.NativeQuery).IsAssignableFrom(query.GetType
				()))
			{
				return true;
			}
			if (typeof(NeoDatis.Odb.Core.Query.NQ.SimpleNativeQuery).IsAssignableFrom(query.GetType
				()))
			{
				return true;
			}
			if (typeof(NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery) == query.GetType(
				) || typeof(NeoDatis.Odb.Impl.Core.Query.Values.ValuesCriteriaQuery) == query.GetType
				())
			{
				return false;
			}
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.QueryTypeNotImplemented
				.AddParameter(query.GetType().FullName));
		}

		public static bool IsCriteriaQuery(NeoDatis.Odb.Core.Query.IQuery query)
		{
			return typeof(NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery).IsAssignableFrom
				(query.GetType());
		}

		public static int[] GetOrderByAttributeIds(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			 classInfo, NeoDatis.Odb.Core.Query.IQuery query)
		{
			string[] fieldNames = query.GetOrderByFieldNames();
			int[] fieldIds = new int[fieldNames.Length];
			for (int i = 0; i < fieldNames.Length; i++)
			{
				fieldIds[i] = classInfo.GetAttributeId(fieldNames[i]);
			}
			return fieldIds;
		}

		/// <summary>Returns a query executor according to the query type</summary>
		/// <param name="query"></param>
		/// <param name="engine"></param>
		/// <param name="instanceBuilder"></param>
		/// <returns></returns>
		public static NeoDatis.Odb.Core.Query.Execution.IQueryExecutor GetQueryExecutor(NeoDatis.Odb.Core.Query.IQuery
			 query, NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine, NeoDatis.Odb.Core.Layers.Layer2.Instance.IInstanceBuilder
			 instanceBuilder)
		{
			if (query.IsPolymorphic())
			{
				return GetMultiClassQueryExecutor(query, engine, instanceBuilder);
			}
			return GetSingleClassQueryExecutor(query, engine, instanceBuilder);
		}

		/// <summary>Return a single class query executor (polymorphic = false)</summary>
		/// <param name="query"></param>
		/// <param name="engine"></param>
		/// <param name="instanceBuilder"></param>
		/// <returns></returns>
		protected static NeoDatis.Odb.Core.Query.Execution.IQueryExecutor GetSingleClassQueryExecutor
			(NeoDatis.Odb.Core.Query.IQuery query, NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine
			 engine, NeoDatis.Odb.Core.Layers.Layer2.Instance.IInstanceBuilder instanceBuilder
			)
		{
			if (typeof(NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery) == query.GetType(
				))
			{
				return new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQueryExecutor(query, engine
					);
			}
			if (typeof(NeoDatis.Odb.Impl.Core.Query.Values.ValuesCriteriaQuery) == query.GetType
				())
			{
				return new NeoDatis.Odb.Impl.Core.Query.Values.ValuesCriteriaQueryExecutor(query, 
					engine);
			}
			if (typeof(NeoDatis.Odb.Core.Query.NQ.NativeQuery).IsAssignableFrom(query.GetType
				()))
			{
				return new NeoDatis.Odb.Impl.Core.Query.NQ.NativeQueryExecutor(query, engine, instanceBuilder
					);
			}
			if (typeof(NeoDatis.Odb.Core.Query.NQ.SimpleNativeQuery).IsAssignableFrom(query.GetType
				()))
			{
				return new NeoDatis.Odb.Impl.Core.Query.NQ.NativeQueryExecutor(query, engine, instanceBuilder
					);
			}
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.QueryTypeNotImplemented
				.AddParameter(query.GetType().FullName));
		}

		/// <summary>Returns a multi class query executor (polymorphic = true)</summary>
		/// <param name="query"></param>
		/// <param name="engine"></param>
		/// <param name="instanceBuilder"></param>
		/// <returns></returns>
		protected static NeoDatis.Odb.Core.Query.Execution.IQueryExecutor GetMultiClassQueryExecutor
			(NeoDatis.Odb.Core.Query.IQuery query, NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine
			 engine, NeoDatis.Odb.Core.Layers.Layer2.Instance.IInstanceBuilder instanceBuilder
			)
		{
			if (typeof(NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery) == query.GetType(
				))
			{
				return new NeoDatis.Odb.Core.Query.Execution.MultiClassGenericQueryExecutor(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQueryExecutor
					(query, engine));
			}
			if (typeof(NeoDatis.Odb.Impl.Core.Query.Values.ValuesCriteriaQuery) == query.GetType
				())
			{
				return new NeoDatis.Odb.Core.Query.Execution.MultiClassGenericQueryExecutor(new NeoDatis.Odb.Impl.Core.Query.Values.ValuesCriteriaQueryExecutor
					(query, engine));
			}
			if (typeof(NeoDatis.Odb.Core.Query.NQ.NativeQuery).IsAssignableFrom(query.GetType
				()))
			{
				return new NeoDatis.Odb.Core.Query.Execution.MultiClassGenericQueryExecutor(new NeoDatis.Odb.Impl.Core.Query.NQ.NativeQueryExecutor
					(query, engine, instanceBuilder));
			}
			if (typeof(NeoDatis.Odb.Core.Query.NQ.SimpleNativeQuery).IsAssignableFrom(query.GetType
				()))
			{
				return new NeoDatis.Odb.Core.Query.Execution.MultiClassGenericQueryExecutor(new NeoDatis.Odb.Impl.Core.Query.NQ.NativeQueryExecutor
					(query, engine, instanceBuilder));
			}
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.QueryTypeNotImplemented
				.AddParameter(query.GetType().FullName));
		}
	}
}
