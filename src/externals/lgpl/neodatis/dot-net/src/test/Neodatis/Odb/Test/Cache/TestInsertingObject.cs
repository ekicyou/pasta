using NUnit.Framework;
using NeoDatis.Odb.Core.Oid;
using NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine;
namespace NeoDatis.Odb.Test.Cache
{
	[TestFixture]
    public class TestInsertingObject : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.IO.IOException"></exception>
		[Test]
        public virtual void Test1()
		{
			NeoDatis.Odb.Core.Transaction.ICache cache = NeoDatis.Odb.Impl.Core.Transaction.CacheFactory
				.GetLocalCache(null, "test");
			string s1 = "ola1";
			string s2 = "ola2";
			string s3 = "ola3";
			cache.StartInsertingObjectWithOid(s1, NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID
				(1), null);
			cache.StartInsertingObjectWithOid(s2, NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID
				(2), null);
			cache.StartInsertingObjectWithOid(s3, NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID
				(3), null);
			AssertTrue(cache.IdOfInsertingObject(s1) != NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.NullObjectId);
			AssertTrue(cache.IdOfInsertingObject(s2) != NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.NullObjectId);
			AssertTrue(cache.IdOfInsertingObject(s3) != NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.NullObjectId);
			cache.EndInsertingObject(s3);
			cache.EndInsertingObject(s2);
			cache.EndInsertingObject(s1);
			AssertTrue(cache.IdOfInsertingObject(s1) == NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.NullObjectId);
			AssertTrue(cache.IdOfInsertingObject(s2) == NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.NullObjectId);
			AssertTrue(cache.IdOfInsertingObject(s3) == NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.NullObjectId);
		}

		/// <exception cref="System.IO.IOException"></exception>
		[Test]
        public virtual void Test2()
		{
			NeoDatis.Odb.Core.Transaction.ICache cache = NeoDatis.Odb.Impl.Core.Transaction.CacheFactory
				.GetLocalCache(null, "temp");
			string s1 = "ola1";
			string s2 = "ola2";
			string s3 = "ola3";
			for (int i = 0; i < 1000 * 3; i += 3)
			{
				cache.StartInsertingObjectWithOid(s1, OIDFactory.BuildObjectOID(i + 1), null);
				cache.StartInsertingObjectWithOid(s2, OIDFactory.BuildObjectOID(i + 2), null);
				cache.StartInsertingObjectWithOid(s3, OIDFactory.BuildObjectOID(i + 3), null);
			}
			AssertEquals(1000, cache.InsertingLevelOf(s1));
			AssertEquals(1000, cache.InsertingLevelOf(s2));
			AssertEquals(1000, cache.InsertingLevelOf(s3));
			for (int i = 0; i < 1000; i++)
			{
				cache.EndInsertingObject(s1);
				cache.EndInsertingObject(s2);
				cache.EndInsertingObject(s3);
			}
			AssertEquals(0, cache.InsertingLevelOf(s1));
			AssertEquals(0, cache.InsertingLevelOf(s2));
			AssertEquals(0, cache.InsertingLevelOf(s3));
			cache.StartInsertingObjectWithOid(s1, OIDFactory.BuildObjectOID(1), null);
			cache.StartInsertingObjectWithOid(s1, OIDFactory.BuildObjectOID(1), null);
			cache.StartInsertingObjectWithOid(s1, OIDFactory.BuildObjectOID(1), null);
			cache.StartInsertingObjectWithOid(s2, OIDFactory.BuildObjectOID(2), null);
			cache.StartInsertingObjectWithOid(s3, OIDFactory.BuildObjectOID(3), null);
			AssertTrue(cache.IdOfInsertingObject(s1) != StorageEngineConstant.NullObjectId);
			AssertTrue(cache.IdOfInsertingObject(s2) != StorageEngineConstant.NullObjectId);
			AssertTrue(cache.IdOfInsertingObject(s3) != StorageEngineConstant.NullObjectId);
			cache.EndInsertingObject(s3);
			cache.EndInsertingObject(s2);
			cache.EndInsertingObject(s1);
			AssertTrue(cache.IdOfInsertingObject(s1) != StorageEngineConstant.NullObjectId);
			AssertTrue(cache.IdOfInsertingObject(s2) == StorageEngineConstant.NullObjectId);
			AssertTrue(cache.IdOfInsertingObject(s3) == StorageEngineConstant.NullObjectId);
		}

		/// <exception cref="System.IO.IOException"></exception>
		[Test]
        public virtual void Test3()
		{
			NeoDatis.Odb.Core.Transaction.ICache cache = NeoDatis.Odb.Impl.Core.Transaction.CacheFactory
				.GetLocalCache(null, "temp");
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci = new NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
				(this.GetType().FullName);
			ci.SetPosition(1);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader oih1 = new NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader
				();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader oih2 = new NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader
				();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader oih3 = new NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader
				();
			oih1.SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(1));
			oih2.SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(10));
			oih3.SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(100));
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoi1 = new NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				(oih1, ci);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoi2 = new NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				(oih2, ci);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoi3 = new NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				(oih3, ci);
			cache.StartReadingObjectInfoWithOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID
				(1), nnoi1);
			cache.StartReadingObjectInfoWithOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID
				(10), nnoi2);
			cache.StartReadingObjectInfoWithOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID
				(100), nnoi3);
			AssertTrue(cache.IsReadingObjectInfoWithOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID
				(1)));
			AssertTrue(cache.IsReadingObjectInfoWithOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID
				(10)));
			AssertTrue(cache.IsReadingObjectInfoWithOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID
				(100)));
			cache.EndReadingObjectInfo(nnoi1.GetOid());
			cache.EndReadingObjectInfo(nnoi2.GetOid());
			cache.EndReadingObjectInfo(nnoi3.GetOid());
			AssertFalse(cache.IsReadingObjectInfoWithOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID
				(1)));
			AssertFalse(cache.IsReadingObjectInfoWithOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID
				(10)));
			AssertFalse(cache.IsReadingObjectInfoWithOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID
				(100)));
		}
	}
}
