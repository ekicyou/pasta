using System;
using NeoDatis.Btree;
using NeoDatis.Btree.Impl;


using NUnit.Framework;
namespace NeoDatis.BTree
{
	
	public class MockBTree : NeoDatis.Btree.IBTree
	{

        #region IBTree Members

        void IBTree.Insert(IComparable key, object value)
        {
            throw new NotImplementedException();
        }

        void IBTree.Split(IBTreeNode parent, IBTreeNode node2Split, int childIndex)
        {
            throw new NotImplementedException();
        }

        object IBTree.Delete(IComparable key, object value)
        {
            throw new NotImplementedException();
        }

        int IBTree.GetDegree()
        {
            throw new NotImplementedException();
        }

        long IBTree.GetSize()
        {
            throw new NotImplementedException();
        }

        int IBTree.GetHeight()
        {
            throw new NotImplementedException();
        }

        IBTreeNode IBTree.GetRoot()
        {
            throw new NotImplementedException();
        }

        IBTreePersister IBTree.GetPersister()
        {
            throw new NotImplementedException();
        }

        void IBTree.SetPersister(IBTreePersister persister)
        {
            throw new NotImplementedException();
        }

        IBTreeNode IBTree.BuildNode()
        {
            throw new NotImplementedException();
        }

        object IBTree.GetId()
        {
            throw new NotImplementedException();
        }

        void IBTree.SetId(object id)
        {
            throw new NotImplementedException();
        }

        void IBTree.Clear()
        {
            throw new NotImplementedException();
        }

        IKeyAndValue IBTree.GetBiggest(IBTreeNode node, bool delete)
        {
            throw new NotImplementedException();
        }

        IKeyAndValue IBTree.GetSmallest(IBTreeNode node, bool delete)
        {
            throw new NotImplementedException();
        }

        System.Collections.IEnumerator IBTree.Iterator<T>(NeoDatis.Odb.Core.OrderByConstants orderBy)
        {
            throw new NotImplementedException();
        }

        #endregion
    }
}