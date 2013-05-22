namespace NeoDatis.Odb.Impl.Core.Btree
{
	/// <summary>Class that persists the BTree and its node into the NeoDatis ODB Database.
	/// 	</summary>
	/// <remarks>Class that persists the BTree and its node into the NeoDatis ODB Database.
	/// 	</remarks>
	/// <author>osmadja</author>
	public class LazyODBBTreePersister : NeoDatis.Btree.IBTreePersister, NeoDatis.Odb.Core.Layers.Layer3.ICommitListener
	{
		public static readonly string LogId = "LazyODBBTreePersister";

		/// <summary>All loaded nodes</summary>
		private System.Collections.Generic.IDictionary<NeoDatis.Odb.OID, object> oids;

		/// <summary>
		/// All modified nodes : the map is used to avoid duplication The key is the
		/// oid, the value is the position is the list
		/// </summary>
		private NeoDatis.Tool.Wrappers.Map.OdbHashMap<object, int> modifiedObjectOids;

		/// <summary>The list is used to keep the order.</summary>
		/// <remarks>
		/// The list is used to keep the order. Deleted object will be replaced by
		/// null value, to keep the positions
		/// </remarks>
		private NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.OID> modifiedObjectOidList;

		/// <summary>The odb interface</summary>
		private NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine;

		/// <summary>The tree we are persisting</summary>
		private NeoDatis.Btree.IBTree tree;

		private static System.Collections.Generic.IDictionary<NeoDatis.Odb.OID, object> smap
			 = null;

		private static System.Collections.Generic.IDictionary<object, int> smodifiedObjects
			 = null;

		public static int nbSaveNodes = 0;

		public static int nbSaveNodesInCache = 0;

		public static int nbSaveTree = 0;

		public static int nbLoadNodes = 0;

		public static int nbLoadTree = 0;

		public static int nbLoadNodesFromCache = 0;

		private int nbPersist;

		public LazyODBBTreePersister(NeoDatis.Odb.ODB odb) : this(NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.Dummy
			.GetEngine(odb))
		{
		}

		public LazyODBBTreePersister(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine
			)
		{
			// See the map strategy performance test at
			// test/org.neodatis.odb.test.performance.TestMapPerf
			// TODO create a boolean value to know if data must be saved on update or
			// only at the end
			oids = new System.Collections.Generic.Dictionary<NeoDatis.Odb.OID, object>();
			modifiedObjectOids = new NeoDatis.Tool.Wrappers.Map.OdbHashMap<object, int>();
			modifiedObjectOidList = new NeoDatis.Tool.Wrappers.List.OdbArrayList<NeoDatis.Odb.OID
				>(500);
			this.engine = engine;
			this.engine.AddCommitListener(this);
			smap = oids;
			smodifiedObjects = modifiedObjectOids;
		}

		/// <summary>Loads a node from its id.</summary>
		/// <remarks>
		/// Loads a node from its id. Tries to get if from memory, if not present
		/// then loads it from odb storage
		/// </remarks>
		/// <param name="id">The id of the nod</param>
		/// <returns>The node with the specific id</returns>
		public virtual NeoDatis.Btree.IBTreeNode LoadNodeById(object id)
		{
			NeoDatis.Odb.OID oid = (NeoDatis.Odb.OID)id;
			// Check if node is in memory
			NeoDatis.Btree.IBTreeNode node = (NeoDatis.Btree.IBTreeNode)oids[oid];
			if (node != null)
			{
				nbLoadNodesFromCache++;
				return node;
			}
			nbLoadNodes++;
			// else load from odb
			try
			{
				if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
				{
					NeoDatis.Tool.DLogger.Debug("Loading node with id " + oid);
				}
				if (oid == null)
				{
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Btree.BTreeError.InvalidIdForBtree
						.AddParameter(oid));
				}
				NeoDatis.Btree.IBTreeNode pn = (NeoDatis.Btree.IBTreeNode)engine.GetObjectFromOid
					(oid);
				pn.SetId(oid);
				if (tree != null)
				{
					pn.SetBTree(tree);
				}
				// Keep the node in memory
				oids.Add(oid, pn);
				return pn;
			}
			catch (System.Exception e)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Btree.BTreeError.InternalError
					, e);
			}
		}

		/// <summary>
		/// saves the bree node Only puts the current node in an 'modified Node' map
		/// to be saved on commit
		/// </summary>
		public virtual object SaveNode(NeoDatis.Btree.IBTreeNode node)
		{
			NeoDatis.Odb.OID oid = null;
			// Here we only save the node if it does not have id,
			// else we just save into the hashmap
			if (node.GetId() == NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.NullObjectId)
			{
				try
				{
					nbSaveNodes++;
					// first get the oid. : -2:it could be any value
					oid = engine.GetObjectWriter().GetIdManager().GetNextObjectId(-2);
					node.SetId(oid);
					oid = engine.Store(oid, node);
					if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
					{
						NeoDatis.Tool.DLogger.Debug("Saved node id " + oid);
					}
					// + " : " +
					// node.toString());
					if (tree != null && node.GetBTree() == null)
					{
						node.SetBTree(tree);
					}
					oids.Add(oid, node);
					return oid;
				}
				catch (System.Exception e)
				{
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Btree.BTreeError.InternalError
						.AddParameter("While saving node"), e);
				}
			}
			nbSaveNodesInCache++;
			oid = (NeoDatis.Odb.OID)node.GetId();
			oids.Add(oid, node);
			AddModifiedOid(oid);
			return oid;
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Close()
		{
			Persist();
			engine.Commit();
			engine.Close();
		}

		public virtual NeoDatis.Btree.IBTree LoadBTree(object id)
		{
			nbLoadTree++;
			NeoDatis.Odb.OID oid = (NeoDatis.Odb.OID)id;
			try
			{
				if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
				{
					NeoDatis.Tool.DLogger.Debug("Loading btree with id " + oid);
				}
				if (oid == NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant.NullObjectId)
				{
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Btree.BTreeError.InvalidIdForBtree
						.AddParameter(NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant.
						NullObjectId));
				}
				tree = (NeoDatis.Btree.IBTree)engine.GetObjectFromOid(oid);
				tree.SetId(oid);
				tree.SetPersister(this);
				NeoDatis.Btree.IBTreeNode root = tree.GetRoot();
				root.SetBTree(tree);
				return tree;
			}
			catch (System.Exception e)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Btree.BTreeError.InternalError
					, e);
			}
		}

		public virtual NeoDatis.Odb.OID SaveBTree(NeoDatis.Btree.IBTree treeToSave)
		{
			nbSaveTree++;
			try
			{
				NeoDatis.Odb.OID oid = (NeoDatis.Odb.OID)treeToSave.GetId();
				if (oid == null)
				{
					// first get the oid. -2 : it could be any value
					oid = engine.GetObjectWriter().GetIdManager().GetNextObjectId(-2);
					treeToSave.SetId(oid);
					oid = engine.Store(oid, treeToSave);
					if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
					{
						NeoDatis.Tool.DLogger.Debug("Saved btree " + treeToSave.GetId() + " with id " + oid
							 + " and  root " + treeToSave.GetRoot());
					}
					if (this.tree == null)
					{
						this.tree = treeToSave;
					}
					oids.Add(oid, treeToSave);
				}
				else
				{
					oids.Add(oid, treeToSave);
					AddModifiedOid(oid);
				}
				return oid;
			}
			catch (System.Exception e)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Btree.BTreeError.InternalError
					, e);
			}
		}

		/// <exception cref="System.IO.IOException"></exception>
		public virtual NeoDatis.Odb.OID GetNextNodeId()
		{
			return engine.GetObjectWriter().GetIdManager().GetNextObjectId(-1);
		}

		public virtual void Persist()
		{
			nbPersist++;
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
			{
				NeoDatis.Tool.DLogger.Debug("persist " + nbPersist + " : Saving " + modifiedObjectOids
					.Count + " objects - " + GetHashCode());
			}
			NeoDatis.Odb.OID oid = null;
			int nbCommited = 0;
			long t0 = 0;
			long t1 = 0;
			int i = 0;
			int size = modifiedObjectOids.Count;
			System.Collections.IEnumerator iterator = modifiedObjectOidList.GetEnumerator();
			while (iterator.MoveNext())
			{
				oid = (NeoDatis.Odb.OID)iterator.Current;
				if (oid != null)
				{
					nbCommited++;
					try
					{
						t0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
						object o = oids[oid];
						engine.Store(o);
						t1 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
					}
					catch (System.Exception e)
					{
						throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Btree.BTreeError.InternalError
							.AddParameter("Error while storing object with oid " + oid), e);
					}
					if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
					{
						NeoDatis.Tool.DLogger.Debug("Committing oid " + oid + " | " + i + "/" + size + " | "
							 + (t1 - t0));
					}
					i++;
				}
			}
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
			{
				NeoDatis.Tool.DLogger.Debug(nbCommited + " commits / " + size);
			}
		}

		public virtual void AfterCommit()
		{
		}

		// nothing to do
		public virtual void BeforeCommit()
		{
			Persist();
			Clear();
		}

		public virtual object DeleteNode(NeoDatis.Btree.IBTreeNode o)
		{
			NeoDatis.Odb.OID oid = engine.Delete(o);
			oids.Remove(oid);
			int position = modifiedObjectOids.Remove2(oid);
            //TODO
			if (position != null)
			{
				// Just replace the element by null, to not modify all the other
				// positions
				modifiedObjectOidList.Set(position, null);
			}
			return o;
		}

		public virtual void SetBTree(NeoDatis.Btree.IBTree tree)
		{
			this.tree = tree;
		}

		public static void ResetCounters()
		{
			nbSaveNodes = 0;
			nbSaveTree = 0;
			nbSaveNodesInCache = 0;
			nbLoadNodes = 0;
			nbLoadTree = 0;
			nbLoadNodesFromCache = 0;
		}

		public static System.Text.StringBuilder Counters()
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder("save nodes=").Append
				(nbSaveNodes).Append(",").Append(nbLoadNodesFromCache).Append(" | save tree=").Append
				(nbSaveTree).Append(" | loadNodes=").Append(nbLoadNodes).Append(",").Append(nbLoadNodesFromCache
				).Append(" | load tree=").Append(nbLoadTree);
			if (smap != null && smodifiedObjects != null)
			{
				buffer.Append(" | map size=").Append(smap.Count).Append(" | modObjects size=").Append
					(smodifiedObjects.Count);
			}
			return buffer;
		}

		public virtual void Clear()
		{
			oids.Clear();
			modifiedObjectOids.Clear();
			modifiedObjectOidList.Clear();
		}

		public virtual void ClearModified()
		{
			modifiedObjectOids.Clear();
			modifiedObjectOidList.Clear();
		}

		public virtual void Flush()
		{
			Persist();
			ClearModified();
		}

		protected virtual void AddModifiedOid(NeoDatis.Odb.OID oid)
		{
			object o = modifiedObjectOids[oid];
			if (o != null)
			{
				// Object is already in the list
				return;
			}
			modifiedObjectOidList.Add(oid);
			// Keep the position of the oid in the list as the value of the map.
			// Used for the delete.
			modifiedObjectOids.Add(oid, modifiedObjectOidList.Count - 1);
		}
	}
}
