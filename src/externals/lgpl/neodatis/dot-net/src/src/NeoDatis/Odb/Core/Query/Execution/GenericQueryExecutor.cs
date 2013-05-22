using System;
namespace NeoDatis.Odb.Core.Query.Execution
{
	/// <summary>
	/// <p>
	/// Generic query executor.
	/// </summary>
	/// <remarks>
	/// <p>
	/// Generic query executor. This class does all the job of iterating in the
	/// object list and call particular query matching to check if the object must be
	/// included in the query result.
	/// </p>
	/// <p>
	/// If the query has index, An execution plan is calculated to optimize the
	/// execution. The query execution plan is calculated by subclasses (using
	/// abstract method getExecutionPlan).
	/// </P>
	/// </remarks>
	public abstract class GenericQueryExecutor : NeoDatis.Odb.Core.Query.Execution.IMultiClassQueryExecutor
	{
		public static readonly string LogId = "GenericQueryExecutor";

		/// <summary>The storage engine</summary>
		protected NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine storageEngine;

		/// <summary>The query being executed</summary>
		protected NeoDatis.Odb.Core.Query.IQuery query;

		/// <summary>The class of the object being fetched</summary>
		protected NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo classInfo;

		/// <summary>The object used to read object data from database</summary>
		protected NeoDatis.Odb.Core.Layers.Layer3.IObjectReader objectReader;

		/// <summary>The current database session</summary>
		protected NeoDatis.Odb.Core.Transaction.ISession session;

		/// <summary>The next object position</summary>
		protected NeoDatis.Odb.OID nextOID;

		/// <summary>A boolean to indicate if query must be ordered</summary>
		private bool queryHasOrderBy;

		/// <summary>The key for ordering</summary>
		private NeoDatis.Tool.Wrappers.OdbComparable orderByKey;

		protected NeoDatis.Odb.OID currentOid;

		protected NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo currentNnoi;

		protected NeoDatis.Odb.Core.Query.Execution.IQueryExecutorCallback callback;

		/// <summary>
		/// Used for multi class executor to indicate not to execute start and end
		/// method of query result action
		/// </summary>
		protected bool executeStartAndEndOfQueryAction;

		public GenericQueryExecutor(NeoDatis.Odb.Core.Query.IQuery query, NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine
			 engine)
		{
			this.query = query;
			this.storageEngine = engine;
			this.objectReader = storageEngine.GetObjectReader();
			this.session = storageEngine.GetSession(true);
			this.callback = NeoDatis.Odb.OdbConfiguration.GetQueryExecutorCallback();
			this.executeStartAndEndOfQueryAction = true;
		}

		public abstract NeoDatis.Odb.Core.Query.Execution.IQueryExecutionPlan GetExecutionPlan
			();

		public abstract void PrepareQuery();

		public abstract System.IComparable ComputeIndexKey(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			 ci, NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoIndex index);

		/// <summary>This can be a NonNAtiveObjectInf or AttributeValuesMap</summary>
		/// <returns></returns>
		public abstract object GetCurrentObjectMetaRepresentation();

		/// <summary>
		/// Check if the object with oid matches the query, returns true
		/// This method must compute the next object oid and the orderBy key if it
		/// exists!
		/// </summary>
		/// <param name="oid">The object position</param>
		/// <param name="loadObjectInfo">
		/// To indicate if object must loaded (when the query indicator
		/// 'in memory' is false, we do not need to load object, only ids)
		/// </param>
		/// <param name="inMemory">To indicate if object must be actually loaded to memory</param>
		public abstract bool MatchObjectWithOid(NeoDatis.Odb.OID oid, bool loadObjectInfo
			, bool inMemory);

		public virtual NeoDatis.Odb.Objects<T> Execute<T>(bool inMemory, int startIndex, 
			int endIndex, bool returnObjects, NeoDatis.Odb.Core.Query.Execution.IMatchingObjectAction
			 queryResultAction)
		{
			if (storageEngine.IsClosed())
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OdbIsClosed
					.AddParameter(storageEngine.GetBaseIdentification().GetIdentification()));
			}
			if (session.IsRollbacked())
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OdbHasBeenRollbacked
					);
			}
			// When used as MultiClass Executor, classInfo is already set
			if (classInfo == null)
			{
				// Class to execute query on
				string fullClassName = NeoDatis.Odb.Core.Query.QueryManager.GetFullClassName(query
					);
				// If the query class does not exist in meta model, return an empty
				// collection
				if (!session.GetMetaModel().ExistClass(fullClassName))
				{
					queryResultAction.Start();
					queryResultAction.End();
					query.SetExecutionPlan(new NeoDatis.Odb.Core.Query.Execution.EmptyExecutionPlan()
						);
					return queryResultAction.GetObjects<T>();
				}
				classInfo = session.GetMetaModel().GetClassInfo(fullClassName, true);
			}
			// Get the query execution plan
			NeoDatis.Odb.Core.Query.Execution.IQueryExecutionPlan plan = GetExecutionPlan();
			plan.Start();
			try
			{
				if (plan.UseIndex() && NeoDatis.Odb.OdbConfiguration.UseIndex())
				{
					return ExecuteUsingIndex<T>(plan.GetIndex(), inMemory, startIndex, endIndex, returnObjects
						, queryResultAction);
				}
				// When query must be applied to a single object
				if (query.IsForSingleOid())
				{
					return ExecuteForOneOid<T>(inMemory, returnObjects, queryResultAction);
				}
				return ExecuteFullScan<T>(inMemory, startIndex, endIndex, returnObjects, queryResultAction
					);
			}
			finally
			{
				plan.End();
			}
		}

		/// <summary>
		/// Query execution full scan
		/// <pre>
		/// startIndex &amp; endIndex
		/// A B C D E F G H I J K L
		/// [1,3] : nb &gt;=1 &amp;&amp; nb&lt;3
		/// 1)
		/// analyze A
		/// nb = 0
		/// nb E [1,3] ? no
		/// r=[]
		/// 2)
		/// analyze B
		/// nb = 1
		/// nb E [1,3] ? yes
		/// r=[B]
		/// 3) analyze C
		/// nb = 2
		/// nb E [1,3] ? yes
		/// r=[B,C]
		/// 4) analyze C
		/// nb = 3
		/// nb E [1,3] ? no and 3&gt; upperBound([1,3]) =&gt; exit
		/// </pre>
		/// </summary>
		/// <param name="inMemory"></param>
		/// <param name="startIndex"></param>
		/// <param name="endIndex"></param>
		/// <param name="returnObjects"></param>
		/// <returns></returns>
		/// <exception cref="System.Exception">System.Exception</exception>
		private NeoDatis.Odb.Objects<T> ExecuteFullScan<T>(bool inMemory, int startIndex, 
			int endIndex, bool returnObjects, NeoDatis.Odb.Core.Query.Execution.IMatchingObjectAction
			 queryResultAction)
		{
			bool objectInRange = false;
			bool objectMatches = false;
			if (storageEngine.IsClosed())
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OdbIsClosed
					.AddParameter(storageEngine.GetBaseIdentification().GetIdentification()));
			}
			long nbObjects = classInfo.GetNumberOfObjects();
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
			{
				NeoDatis.Tool.DLogger.Debug("loading " + nbObjects + " instance(s) of " + classInfo
					.GetFullClassName());
			}
			if (ExecuteStartAndEndOfQueryAction())
			{
				queryResultAction.Start();
			}
			NeoDatis.Odb.OID currentOID = null;
			NeoDatis.Odb.OID prevOID = null;
			// TODO check if all instances are in the cache! and then load from the
			// cache
			nextOID = classInfo.GetCommitedZoneInfo().first;
			if (nbObjects > 0 && nextOID == null)
			{
				// This means that some changes have not been commited!
				// Take next position from uncommited zone
				nextOID = classInfo.GetUncommittedZoneInfo().first;
			}
			PrepareQuery();
			if (query != null)
			{
				queryHasOrderBy = query.HasOrderBy();
			}
			bool monitorMemory = NeoDatis.Odb.OdbConfiguration.IsMonitoringMemory();
			// used when startIndex and endIndex are not negative
			int nbObjectsInResult = 0;
			for (int i = 0; i < nbObjects; i++)
			{
                //Console.WriteLine(i);
				if (monitorMemory && i % 10000 == 0)
				{
					NeoDatis.Odb.Impl.Tool.MemoryMonitor.DisplayCurrentMemory(string.Empty + (i + 1), 
						true);
				}
				// Reset the order by key
				orderByKey = null;
				objectMatches = false;
				prevOID = currentOID;
				currentOID = nextOID;
				// This is an error
				if (currentOID == null)
				{
					if (NeoDatis.Odb.OdbConfiguration.ThrowExceptionWhenInconsistencyFound())
					{
						throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.NullNextObjectOid
							.AddParameter(classInfo.GetFullClassName()).AddParameter(i).AddParameter(nbObjects
							).AddParameter(prevOID));
					}
					break;
				}
				// If there is an endIndex condition
				if (endIndex != -1 && nbObjectsInResult >= endIndex)
				{
					break;
				}
				// If there is a startIndex condition
				if (startIndex != -1 && nbObjectsInResult < startIndex)
				{
					objectInRange = false;
				}
				else
				{
					objectInRange = true;
				}
				// There is no query
				if (!inMemory && query == null)
				{
					nbObjectsInResult++;
					// keep object position if we must
					if (objectInRange)
					{
						orderByKey = BuildOrderByKey(currentNnoi);
						// TODO Where is the key for order by
						queryResultAction.ObjectMatch(nextOID, orderByKey);
					}
					nextOID = objectReader.GetNextObjectOID(currentOID);
				}
				else
				{
					objectMatches = MatchObjectWithOid(currentOID, returnObjects, inMemory);
					if (objectMatches)
					{
						nbObjectsInResult++;
						if (objectInRange)
						{
							if (queryHasOrderBy)
							{
								orderByKey = BuildOrderByKey(GetCurrentObjectMetaRepresentation());
							}
							queryResultAction.ObjectMatch(currentOID, GetCurrentObjectMetaRepresentation(), orderByKey
								);
							if (callback != null)
							{
								callback.ReadingObject(i, -1);
							}
						}
					}
				}
			}
			if (ExecuteStartAndEndOfQueryAction())
			{
				queryResultAction.End();
			}
			return queryResultAction.GetObjects<T>();
		}

		/// <summary>Execute query using index</summary>
		/// <param name="index"></param>
		/// <param name="inMemory"></param>
		/// <param name="startIndex"></param>
		/// <param name="endIndex"></param>
		/// <param name="returnObjects"></param>
		/// <returns></returns>
		/// <exception cref="System.Exception">System.Exception</exception>
		private NeoDatis.Odb.Objects<T> ExecuteUsingIndex<T>(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoIndex
			 index, bool inMemory, int startIndex, int endIndex, bool returnObjects, NeoDatis.Odb.Core.Query.Execution.IMatchingObjectAction
			 queryResultAction)
		{
			// Index that have not been used yet do not have persister!
			if (index.GetBTree().GetPersister() == null)
			{
				index.GetBTree().SetPersister(new NeoDatis.Odb.Impl.Core.Btree.LazyODBBTreePersister
					(storageEngine));
			}
			bool objectMatches = false;
			long nbObjects = classInfo.GetNumberOfObjects();
			long btreeSize = index.GetBTree().GetSize();
			// the two values should be equal
			if (nbObjects != btreeSize)
			{
				NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci = storageEngine.GetSession(true
					).GetMetaModel().GetClassInfoFromId(index.GetClassInfoId());
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.IndexIsCorrupted
					.AddParameter(index.GetName()).AddParameter(ci.GetFullClassName()).AddParameter(
					nbObjects).AddParameter(btreeSize));
			}
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
			{
				NeoDatis.Tool.DLogger.Debug("loading " + nbObjects + " instance(s) of " + classInfo
					.GetFullClassName());
			}
			if (ExecuteStartAndEndOfQueryAction())
			{
				queryResultAction.Start();
			}
			PrepareQuery();
			if (query != null)
			{
				queryHasOrderBy = query.HasOrderBy();
			}
			NeoDatis.Btree.IBTree tree = index.GetBTree();
			bool isUnique = index.IsUnique();
			// Iterator iterator = new BTreeIterator(tree,
			// OrderByConstants.ORDER_BY_ASC);
			System.IComparable key = ComputeIndexKey(classInfo, index);
			System.Collections.IList list = null;
			// If index is unique, get the object
			if (isUnique)
			{
				NeoDatis.Btree.IBTreeSingleValuePerKey treeSingle = (NeoDatis.Btree.IBTreeSingleValuePerKey
					)tree;
				object o = treeSingle.Search(key);
				if (o != null)
				{
					list = new System.Collections.ArrayList();
					list.Add(o);
				}
			}
			else
			{
				NeoDatis.Btree.IBTreeMultipleValuesPerKey treeMultiple = (NeoDatis.Btree.IBTreeMultipleValuesPerKey
					)tree;
				list = treeMultiple.Search(key);
			}
			if (list != null)
			{
				System.Collections.IEnumerator iterator = list.GetEnumerator();
				while (iterator.MoveNext())
				{
					NeoDatis.Odb.OID oid = (NeoDatis.Odb.OID)iterator.Current;
					// FIXME Why calling this method
					long position = objectReader.GetObjectPositionFromItsOid(oid, true, true);
					orderByKey = null;
					objectMatches = MatchObjectWithOid(oid, returnObjects, inMemory);
					if (objectMatches)
					{
						queryResultAction.ObjectMatch(oid, GetCurrentObjectMetaRepresentation(), orderByKey
							);
					}
				}
				queryResultAction.End();
				return queryResultAction.GetObjects<T>();
			}
			if (ExecuteStartAndEndOfQueryAction())
			{
				queryResultAction.End();
			}
			return queryResultAction.GetObjects<T>();
		}

		/// <summary>Execute query using index</summary>
		/// <param name="inMemory"></param>
		/// <param name="returnObjects"></param>
		/// <returns></returns>
		/// <exception cref="System.Exception">System.Exception</exception>
		private NeoDatis.Odb.Objects<T> ExecuteForOneOid<T>(bool inMemory, bool returnObjects
			, NeoDatis.Odb.Core.Query.Execution.IMatchingObjectAction queryResultAction)
		{
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
			{
				NeoDatis.Tool.DLogger.Debug("loading Object with oid " + query.GetOidOfObjectToQuery
					() + " - class " + classInfo.GetFullClassName());
			}
			if (ExecuteStartAndEndOfQueryAction())
			{
				queryResultAction.Start();
			}
			PrepareQuery();
			NeoDatis.Odb.OID oid = query.GetOidOfObjectToQuery();
			// FIXME Why calling this method
			long position = objectReader.GetObjectPositionFromItsOid(oid, true, true);
			bool objectMatches = MatchObjectWithOid(oid, returnObjects, inMemory);
			queryResultAction.ObjectMatch(oid, GetCurrentObjectMetaRepresentation(), orderByKey
				);
			queryResultAction.End();
			return queryResultAction.GetObjects<T>();
		}

		/// <summary>TODO very bad.</summary>
		/// <remarks>TODO very bad. Should remove the instanceof</remarks>
		/// <param name="@object"></param>
		/// <returns></returns>
		public virtual NeoDatis.Tool.Wrappers.OdbComparable BuildOrderByKey(object @object
			)
		{
			if (@object is NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap)
			{
				return BuildOrderByKey((NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap)@object
					);
			}
			return BuildOrderByKey((NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo)
				@object);
		}

		public virtual NeoDatis.Tool.Wrappers.OdbComparable BuildOrderByKey(NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 nnoi)
		{
			// TODO cache the attributes ids to compute them only once
			return NeoDatis.Odb.Core.Query.Execution.IndexTool.BuildIndexKey("OrderBy", nnoi, 
				NeoDatis.Odb.Core.Query.QueryManager.GetOrderByAttributeIds(classInfo, query));
		}

		public virtual NeoDatis.Tool.Wrappers.OdbComparable BuildOrderByKey(NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap
			 values)
		{
			return NeoDatis.Odb.Core.Query.Execution.IndexTool.BuildIndexKey("OrderBy", values
				, query.GetOrderByFieldNames());
		}

		public virtual bool ExecuteStartAndEndOfQueryAction()
		{
			return executeStartAndEndOfQueryAction;
		}

		public virtual void SetExecuteStartAndEndOfQueryAction(bool yes)
		{
			this.executeStartAndEndOfQueryAction = yes;
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine GetStorageEngine()
		{
			return storageEngine;
		}

		public virtual NeoDatis.Odb.Core.Query.IQuery GetQuery()
		{
			return query;
		}

		public virtual void SetClassInfo(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo classInfo
			)
		{
			this.classInfo = classInfo;
		}
	}
}
