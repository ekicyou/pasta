using NUnit.Framework;
namespace NeoDatis.Odb.Test.Intropector
{
	public class InstrospectorTest : NeoDatis.Odb.Test.ODBTest
	{
		internal static NeoDatis.Odb.Core.Layers.Layer1.Introspector.IClassIntrospector classIntrospector
			 = NeoDatis.Odb.OdbConfiguration.GetCoreProvider().GetClassIntrospector();

		/// <exception cref="System.Exception"></exception>
		public override void SetUp()
		{
			base.SetUp();
			NeoDatis.Odb.OdbConfiguration.GetCoreProvider().GetClientServerSessionManager().AddSession
				(new NeoDatis.Odb.Core.Mock.MockSession("test"));
		}

		public override void TearDown()
		{
			NeoDatis.Odb.OdbConfiguration.GetCoreProvider().GetClientServerSessionManager().RemoveSession
				("test");
		}

		[Test]
        public virtual void TestClassInfo()
		{
			NeoDatis.Odb.Test.VO.Login.User user = new NeoDatis.Odb.Test.VO.Login.User("olivier smadja"
				, "olivier@neodatis.com", new NeoDatis.Odb.Test.VO.Login.Profile("operator", new 
				NeoDatis.Odb.Test.VO.Login.Function("login")));
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoList classInfoList = classIntrospector
				.Introspect(user.GetType(), true);
			AssertEquals(user.GetType().FullName, classInfoList.GetMainClassInfo().GetFullClassName
				());
			AssertEquals(3, classInfoList.GetMainClassInfo().GetAttributes().Count);
			AssertEquals(2, classInfoList.GetClassInfos().Count);
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestInstanceInfo()
		{
			NeoDatis.Odb.Test.VO.Login.User user = new NeoDatis.Odb.Test.VO.Login.User("olivier smadja"
				, "olivier@neodatis.com", new NeoDatis.Odb.Test.VO.Login.Profile("operator", new 
				NeoDatis.Odb.Test.VO.Login.Function("login")));
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci = classIntrospector.Introspect(
				user.GetType(), true).GetMainClassInfo();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo instanceInfo = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.LocalObjectIntrospector(new 
				NeoDatis.Odb.Core.Mock.MockStorageEngine()).GetMetaRepresentation(user, ci, true
				, null, new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.DefaultInstrospectionCallbackForStore
				(null, null, false));
			AssertEquals(user.GetType().FullName, instanceInfo.GetClassInfo().GetFullClassName
				());
			AssertEquals("olivier smadja", instanceInfo.GetAttributeValueFromId(ci.GetAttributeId
				("name")).ToString());
			AssertEquals(typeof(NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo), 
				instanceInfo.GetAttributeValueFromId(ci.GetAttributeId("name")).GetType());
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestInstanceInfo2()
		{
			NeoDatis.Odb.Test.VO.Login.User user = new NeoDatis.Odb.Test.VO.Login.User("olivier smadja"
				, "olivier@neodatis.com", new NeoDatis.Odb.Test.VO.Login.Profile("operator", new 
				NeoDatis.Odb.Test.VO.Login.Function("login")));
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci = classIntrospector.Introspect(
				user.GetType(), true).GetMainClassInfo();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo instanceInfo = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.LocalObjectIntrospector(new 
				NeoDatis.Odb.Core.Mock.MockStorageEngine()).GetMetaRepresentation(user, ci, true
				, null, new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.DefaultInstrospectionCallbackForStore
				(null, null, false));
			AssertEquals(instanceInfo.GetClassInfo().GetFullClassName(), user.GetType().FullName
				);
			AssertEquals(instanceInfo.GetAttributeValueFromId(ci.GetAttributeId("name")).ToString
				(), "olivier smadja");
		}

		[Test]
        public virtual void TestSuperClass()
		{
			System.Type collection = typeof(System.Collections.ICollection);
			System.Type arrayList = typeof(System.Collections.ArrayList);
			System.Type @string = typeof(string);
			bool b1 = collection.IsAssignableFrom(arrayList);
			bool b2 = arrayList.IsAssignableFrom(collection);
			bool b3 = collection.IsAssignableFrom(@string);
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestCompareCollection1()
		{
			NeoDatis.Odb.Test.VO.Login.User user = new NeoDatis.Odb.Test.VO.Login.User("olivier smadja"
				, "olivier@neodatis.com", new NeoDatis.Odb.Test.VO.Login.Profile("operator", new 
				NeoDatis.Odb.Test.VO.Login.Function("login")));
			NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.IObjectInfoComparator comparator = new 
				NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Compare.ObjectInfoComparator();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci = classIntrospector.Introspect(
				user.GetType(), true).GetMainClassInfo();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo instanceInfo = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.LocalObjectIntrospector(new 
				NeoDatis.Odb.Core.Mock.MockStorageEngine()).GetMetaRepresentation(user, ci, true
				, null, new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.DefaultInstrospectionCallbackForStore
				(null, null, false));
			// Sets attributes offsets - this is normally done by reading then from
			// disk, but in this junit,
			// we must set them manually
			long[] offsets = new long[] { 1L, 2L, 3L };
			int[] ids = new int[] { 1, 2, 3 };
			instanceInfo.GetHeader().SetAttributesIdentification(offsets);
			instanceInfo.GetHeader().SetAttributesIds(ids);
			instanceInfo.GetHeader().SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(1
				));
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoiProfile = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)instanceInfo.GetAttributeValueFromId(2);
			nnoiProfile.SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(2));
			user.SetName("Olivier Smadja");
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo instanceInfo3 = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.LocalObjectIntrospector(new 
				NeoDatis.Odb.Core.Mock.MockStorageEngine()).GetMetaRepresentation(user, ci, true
				, null, new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.DefaultInstrospectionCallbackForStore
				(null, null, false));
			instanceInfo3.GetHeader().SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(
				1));
			nnoiProfile = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo)instanceInfo3
				.GetAttributeValueFromId(2);
			nnoiProfile.SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(2));
			AssertTrue(comparator.HasChanged(instanceInfo, instanceInfo3));
			AssertEquals(1, comparator.GetNbChanges());
			AssertEquals(1, comparator.GetChangedAttributeActions().Count);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.ChangedNativeAttributeAction cnaa = 
				(NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.ChangedNativeAttributeAction)comparator
				.GetChangedAttributeActions()[0];
			AssertEquals("Olivier Smadja", cnaa.GetNoiWithNewValue().GetObject());
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestCompareCollection11()
		{
			NeoDatis.Odb.Test.VO.Login.User user = new NeoDatis.Odb.Test.VO.Login.User("olivier smadja"
				, "olivier@neodatis.com", new NeoDatis.Odb.Test.VO.Login.Profile("operator", new 
				NeoDatis.Odb.Test.VO.Login.Function("login")));
			NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.IObjectInfoComparator comparator = new 
				NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Compare.ObjectInfoComparator();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci = classIntrospector.Introspect(
				user.GetType(), true).GetMainClassInfo();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo instanceInfo = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.LocalObjectIntrospector(new 
				NeoDatis.Odb.Core.Mock.MockStorageEngine()).GetMetaRepresentation(user, ci, true
				, null, new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.DefaultInstrospectionCallbackForStore
				(null, null, false));
			// Sets attributes offsets - this is normally done by reading then from
			// disk, but in this junit,
			// we must set them manually
			long[] offsets = new long[] { 1L, 2L, 3L };
			int[] ids = new int[] { 1, 2, 3 };
			instanceInfo.GetHeader().SetAttributesIdentification(offsets);
			instanceInfo.GetHeader().SetAttributesIds(ids);
			instanceInfo.GetHeader().SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(1
				));
			object o = instanceInfo.GetAttributeValueFromId(2);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoiProfile = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)o;
			nnoiProfile.SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(2));
			user.SetName("Olivier Smadja");
			user.SetEmail("olivier@neodatis.org");
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo instanceInfo3 = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.LocalObjectIntrospector(new 
				NeoDatis.Odb.Core.Mock.MockStorageEngine()).GetMetaRepresentation(user, ci, true
				, null, new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.DefaultInstrospectionCallbackForStore
				(null, null, false));
			instanceInfo3.GetHeader().SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(
				1));
			nnoiProfile = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo)instanceInfo3
				.GetAttributeValueFromId(2);
			nnoiProfile.SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(2));
			AssertTrue(comparator.HasChanged(instanceInfo, instanceInfo3));
			AssertEquals(2, comparator.GetNbChanges());
			AssertEquals(2, comparator.GetChangedAttributeActions().Count);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.ChangedNativeAttributeAction cnaa = 
				(NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.ChangedNativeAttributeAction)comparator
				.GetChangedAttributeActions()[0];
			AssertEquals("Olivier Smadja", cnaa.GetNoiWithNewValue().GetObject());
			cnaa = (NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.ChangedNativeAttributeAction
				)comparator.GetChangedAttributeActions()[1];
			AssertEquals("olivier@neodatis.org", cnaa.GetNoiWithNewValue().GetObject());
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestCompareCollection2()
		{
			NeoDatis.Odb.Test.VO.Login.User user = new NeoDatis.Odb.Test.VO.Login.User("olivier smadja"
				, "olivier@neodatis.com", new NeoDatis.Odb.Test.VO.Login.Profile("operator", new 
				NeoDatis.Odb.Test.VO.Login.Function("login")));
			NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.IObjectInfoComparator comparator = new 
				NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Compare.ObjectInfoComparator();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci = classIntrospector.Introspect(
				user.GetType(), true).GetMainClassInfo();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo instanceInfo = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.LocalObjectIntrospector(new 
				NeoDatis.Odb.Core.Mock.MockStorageEngine()).GetMetaRepresentation(user, ci, true
				, null, new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.DefaultInstrospectionCallbackForStore
				(null, null, false));
			// Sets attributes offsets - this is normally done by reading them from
			// disk, but in this junit,
			// we must set them manually
			long[] offsets = new long[] { 1L, 2L, 3L };
			int[] ids = new int[] { 1, 2, 3 };
			instanceInfo.GetHeader().SetAttributesIdentification(offsets);
			instanceInfo.GetHeader().SetAttributesIds(ids);
			instanceInfo.GetHeader().SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(1
				));
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoiProfile = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)instanceInfo.GetAttributeValueFromId(2);
			nnoiProfile.SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(2));
			user.SetName(null);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo instanceInfo3 = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.LocalObjectIntrospector(new 
				NeoDatis.Odb.Core.Mock.MockStorageEngine()).GetMetaRepresentation(user, ci, true
				, null, new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.DefaultInstrospectionCallbackForStore
				(null, null, false));
			instanceInfo3.GetHeader().SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(
				1));
			nnoiProfile = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo)instanceInfo3
				.GetAttributeValueFromId(2);
			nnoiProfile.SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(2));
			AssertTrue(comparator.HasChanged(instanceInfo, instanceInfo3));
			AssertEquals(1, comparator.GetNbChanges());
			AssertEquals(1, comparator.GetChangedAttributeActions().Count);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.ChangedNativeAttributeAction cnaa = 
				(NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.ChangedNativeAttributeAction)comparator
				.GetChangedAttributeActions()[0];
			AssertEquals(null, cnaa.GetNoiWithNewValue().GetObject());
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestCompareCollection3CollectionContentChange()
		{
			NeoDatis.Odb.Test.VO.Login.Function function = new NeoDatis.Odb.Test.VO.Login.Function
				("login");
			NeoDatis.Odb.Test.VO.Login.User user = new NeoDatis.Odb.Test.VO.Login.User("olivier smadja"
				, "olivier@neodatis.com", new NeoDatis.Odb.Test.VO.Login.Profile("operator", function
				));
			NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.IObjectInfoComparator comparator = new 
				NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Compare.ObjectInfoComparator();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci = classIntrospector.Introspect(
				user.GetType(), true).GetMainClassInfo();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo instanceInfo = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.LocalObjectIntrospector(new 
				NeoDatis.Odb.Core.Mock.MockStorageEngine()).GetMetaRepresentation(user, ci, true
				, null, new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.DefaultInstrospectionCallbackForStore
				(null, null, false));
			// Sets attributes offsets - this is normally done by reading then from
			// disk, but in this junit,
			// we must set them manually
			long[] offsets = new long[] { 1L, 2L, 3L };
			int[] ids = new int[] { 1, 2, 3 };
			instanceInfo.GetHeader().SetAttributesIdentification(offsets);
			instanceInfo.GetHeader().SetAttributesIds(ids);
			instanceInfo.GetHeader().SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(1
				));
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoiProfile = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)instanceInfo.GetAttributeValueFromId(2);
			nnoiProfile.SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(2));
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoi = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)instanceInfo.GetAttributeValueFromId(ci.GetAttributeId("profile"));
			nnoi.GetHeader().SetAttributesIdentification(offsets);
			nnoi.GetHeader().SetAttributesIds(ids);
			nnoi.SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(2));
			NeoDatis.Odb.Core.Layers.Layer2.Meta.CollectionObjectInfo nnoi2 = (NeoDatis.Odb.Core.Layers.Layer2.Meta.CollectionObjectInfo
				)nnoi.GetAttributeValueFromId(nnoi.GetClassInfo().GetAttributeId("functions"));
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoi3 = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)nnoi2.GetCollection().GetEnumerator().Current;
			nnoi3.GetHeader().SetAttributesIdentification(offsets);
			nnoi3.GetHeader().SetAttributesIds(ids);
			function.SetName("login function");
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo instanceInfo3 = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.LocalObjectIntrospector(new 
				NeoDatis.Odb.Core.Mock.MockStorageEngine()).GetMetaRepresentation(user, ci, true
				, null, new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.DefaultInstrospectionCallbackForStore
				(null, null, false));
			instanceInfo3.GetHeader().SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(
				1));
			nnoiProfile = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo)instanceInfo3
				.GetAttributeValueFromId(2);
			nnoiProfile.SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(2));
			AssertTrue(comparator.HasChanged(instanceInfo, instanceInfo3));
			AssertEquals(1, comparator.GetNbChanges());
			NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.ChangedNativeAttributeAction cnaa = 
				(NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.ChangedNativeAttributeAction)comparator
				.GetChangedAttributeActions()[0];
			AssertEquals(1, comparator.GetChangedAttributeActions().Count);
			AssertEquals(function.GetName(), cnaa.GetNoiWithNewValue().GetObject());
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestCompareCollection4CollectionContentChange()
		{
			if (!testNewFeature)
			{
				return;
			}
			NeoDatis.Odb.Test.VO.Login.Function function = new NeoDatis.Odb.Test.VO.Login.Function
				("login");
			NeoDatis.Odb.Test.VO.Login.User user = new NeoDatis.Odb.Test.VO.Login.User("olivier smadja"
				, "olivier@neodatis.com", new NeoDatis.Odb.Test.VO.Login.Profile("operator", function
				));
			NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.IObjectInfoComparator comparator = new 
				NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Compare.ObjectInfoComparator();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci = classIntrospector.Introspect(
				user.GetType(), true).GetMainClassInfo();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo instanceInfo = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.LocalObjectIntrospector(new 
				NeoDatis.Odb.Core.Mock.MockStorageEngine()).GetMetaRepresentation(user, ci, true
				, null, new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.DefaultInstrospectionCallbackForStore
				(null, null, false));
			// Sets attributes offsets - this is normally done by reading then from
			// disk, but in this junit,
			// we must set them manually
			long[] offsets = new long[] { 1L, 2L, 3L };
			int[] ids = new int[] { 1, 2, 3 };
			instanceInfo.GetHeader().SetAttributesIdentification(offsets);
			instanceInfo.GetHeader().SetAttributesIds(ids);
			instanceInfo.GetHeader().SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(1
				));
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoiProfile = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)instanceInfo.GetAttributeValueFromId(2);
			nnoiProfile.SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(2));
			function.SetName(null);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo instanceInfo3 = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.LocalObjectIntrospector(new 
				NeoDatis.Odb.Core.Mock.MockStorageEngine()).GetMetaRepresentation(user, ci, true
				, null, new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.DefaultInstrospectionCallbackForStore
				(null, null, false));
			instanceInfo3.GetHeader().SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(
				1));
			nnoiProfile = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo)instanceInfo3
				.GetAttributeValueFromId(2);
			nnoiProfile.SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(2));
			AssertTrue(comparator.HasChanged(instanceInfo, instanceInfo3));
			AssertEquals(1, comparator.GetNbChanges());
			NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.ChangedNativeAttributeAction cnaa = 
				(NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.ChangedNativeAttributeAction)comparator
				.GetChangedAttributeActions()[0];
			AssertEquals(1, comparator.GetChangedAttributeActions().Count);
			AssertEquals(function.GetName(), cnaa.GetNoiWithNewValue().GetObject());
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestCompareCollection5()
		{
			NeoDatis.Odb.Test.VO.Login.Function function = new NeoDatis.Odb.Test.VO.Login.Function
				("login");
			NeoDatis.Odb.Test.VO.Login.Profile profile = new NeoDatis.Odb.Test.VO.Login.Profile
				("operator", function);
			NeoDatis.Odb.Test.VO.Login.User user = new NeoDatis.Odb.Test.VO.Login.User("olivier smadja"
				, "olivier@neodatis.com", profile);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.IObjectInfoComparator comparator = new 
				NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Compare.ObjectInfoComparator();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci = classIntrospector.Introspect(
				user.GetType(), true).GetMainClassInfo();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo instanceInfo = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.LocalObjectIntrospector(new 
				NeoDatis.Odb.Core.Mock.MockStorageEngine()).GetMetaRepresentation(user, ci, true
				, null, new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.DefaultInstrospectionCallbackForStore
				(null, null, false));
			// Sets attributes offsets - this is normally done by reading then from
			// disk, but in this junit,
			// we must set them manually
			long[] offsets = new long[] { 1L, 2L, 3L };
			int[] ids = new int[] { 1, 2, 3 };
			instanceInfo.GetHeader().SetAttributesIdentification(offsets);
			instanceInfo.GetHeader().SetAttributesIds(ids);
			instanceInfo.GetHeader().SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(1
				));
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoiProfile = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)instanceInfo.GetAttributeValueFromId(2);
			nnoiProfile.SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(2));
			profile.GetFunctions().Add(new NeoDatis.Odb.Test.VO.Login.Function("logout"));
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo instanceInfo3 = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.LocalObjectIntrospector(new 
				NeoDatis.Odb.Core.Mock.MockStorageEngine()).GetMetaRepresentation(user, ci, true
				, null, new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.DefaultInstrospectionCallbackForStore
				(null, null, false));
			instanceInfo3.GetHeader().SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(
				1));
			nnoiProfile = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo)instanceInfo3
				.GetAttributeValueFromId(2);
			nnoiProfile.SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(2));
			AssertTrue(comparator.HasChanged(instanceInfo, instanceInfo3));
			AssertEquals(1, comparator.GetNbChanges());
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoi = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)comparator.GetChangedObjectMetaRepresentation(0);
			AssertEquals(2, ((System.Collections.IList)nnoi.GetValueOf("functions")).Count);
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestCompareCollection6()
		{
			NeoDatis.Odb.Test.VO.Login.Function function = new NeoDatis.Odb.Test.VO.Login.Function
				("login");
			NeoDatis.Odb.Test.VO.Login.Profile profile = new NeoDatis.Odb.Test.VO.Login.Profile
				("operator", function);
			NeoDatis.Odb.Test.VO.Login.User user = new NeoDatis.Odb.Test.VO.Login.User("olivier smadja"
				, "olivier@neodatis.com", profile);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.IObjectInfoComparator comparator = new 
				NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Compare.ObjectInfoComparator();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci = classIntrospector.Introspect(
				user.GetType(), true).GetMainClassInfo();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo instanceInfo = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.LocalObjectIntrospector(new 
				NeoDatis.Odb.Core.Mock.MockStorageEngine()).GetMetaRepresentation(user, ci, true
				, null, new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.DefaultInstrospectionCallbackForStore
				(null, null, false));
			// Sets attributes offsets - this is normally done by reading then from
			// disk, but in this junit,
			// we must set them manually
			long[] offsets = new long[] { 1L, 2L, 3L };
			int[] ids = new int[] { 1, 2, 3 };
			instanceInfo.GetHeader().SetAttributesIdentification(offsets);
			instanceInfo.GetHeader().SetAttributesIds(ids);
			instanceInfo.GetHeader().SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(1
				));
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoiProfile = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)instanceInfo.GetAttributeValueFromId(2);
			nnoiProfile.SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(2));
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoi = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)instanceInfo.GetAttributeValueFromId(ci.GetAttributeId("profile"));
			nnoi.GetHeader().SetAttributesIdentification(offsets);
			nnoi.GetHeader().SetAttributesIds(ids);
			profile.SetName("ope");
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo instanceInfo3 = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.LocalObjectIntrospector(new 
				NeoDatis.Odb.Core.Mock.MockStorageEngine()).GetMetaRepresentation(user, ci, true
				, null, new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.DefaultInstrospectionCallbackForStore
				(null, null, false));
			instanceInfo3.GetHeader().SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(
				1));
			nnoiProfile = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo)instanceInfo3
				.GetAttributeValueFromId(2);
			nnoiProfile.SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(2));
			AssertTrue(comparator.HasChanged(instanceInfo, instanceInfo3));
			AssertEquals(1, comparator.GetNbChanges());
			NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.ChangedNativeAttributeAction cnaa = 
				(NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.ChangedNativeAttributeAction)comparator
				.GetChangedAttributeActions()[0];
			AssertEquals(1, comparator.GetChangedAttributeActions().Count);
			AssertEquals(profile.GetName(), cnaa.GetNoiWithNewValue().GetObject());
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestCompareCollection7()
		{
			NeoDatis.Odb.Test.VO.Login.Function function = new NeoDatis.Odb.Test.VO.Login.Function
				("login");
			NeoDatis.Odb.Test.VO.Login.Profile profile = new NeoDatis.Odb.Test.VO.Login.Profile
				("operator", function);
			NeoDatis.Odb.Test.VO.Login.User user = new NeoDatis.Odb.Test.VO.Login.User("olivier smadja"
				, "olivier@neodatis.com", profile);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.IObjectInfoComparator comparator = new 
				NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Compare.ObjectInfoComparator();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci = classIntrospector.Introspect(
				user.GetType(), true).GetMainClassInfo();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo instanceInfo = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.LocalObjectIntrospector(new 
				NeoDatis.Odb.Core.Mock.MockStorageEngine()).GetMetaRepresentation(user, ci, true
				, null, new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.DefaultInstrospectionCallbackForStore
				(null, null, false));
			// Sets attributes offsets - this is normally done by reading then from
			// disk, but in this junit,
			// we must set them manually
			long[] offsets = new long[] { 1L, 2L, 3L };
			int[] ids = new int[] { 1, 2, 3 };
			instanceInfo.GetHeader().SetAttributesIdentification(offsets);
			instanceInfo.GetHeader().SetAttributesIds(ids);
			instanceInfo.GetHeader().SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(1
				));
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoiProfile = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)instanceInfo.GetAttributeValueFromId(2);
			nnoiProfile.SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(2));
			// / Set the same name
			profile.SetName("operator");
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo instanceInfo3 = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.LocalObjectIntrospector(new 
				NeoDatis.Odb.Core.Mock.MockStorageEngine()).GetMetaRepresentation(user, ci, true
				, null, new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.DefaultInstrospectionCallbackForStore
				(null, null, false));
			instanceInfo3.GetHeader().SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(
				1));
			nnoiProfile = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo)instanceInfo3
				.GetAttributeValueFromId(2);
			nnoiProfile.SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(2));
			AssertFalse(comparator.HasChanged(instanceInfo, instanceInfo3));
			AssertEquals(0, comparator.GetNbChanges());
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestCompareCollection8()
		{
			NeoDatis.Odb.Test.VO.Login.Function function = new NeoDatis.Odb.Test.VO.Login.Function
				("login");
			NeoDatis.Odb.Test.VO.Login.Profile profile = new NeoDatis.Odb.Test.VO.Login.Profile
				("operator", function);
			NeoDatis.Odb.Test.VO.Login.User user = new NeoDatis.Odb.Test.VO.Login.User("olivier smadja"
				, "olivier@neodatis.com", profile);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.IObjectInfoComparator comparator = new 
				NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Compare.ObjectInfoComparator();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci = classIntrospector.Introspect(
				user.GetType(), true).GetMainClassInfo();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo instanceInfo = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.LocalObjectIntrospector(new 
				NeoDatis.Odb.Core.Mock.MockStorageEngine()).GetMetaRepresentation(user, ci, true
				, null, new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.DefaultInstrospectionCallbackForStore
				(null, null, false));
			// Sets attributes offsets - this is normally done by reading then from
			// disk, but in this junit,
			// we must set them manually
			long[] offsets = new long[] { 1L, 2L, 3L };
			int[] ids = new int[] { 1, 2, 3 };
			instanceInfo.GetHeader().SetAttributesIdentification(offsets);
			instanceInfo.GetHeader().SetAttributesIds(ids);
			user.SetProfile(null);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo instanceInfo3 = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.LocalObjectIntrospector(new 
				NeoDatis.Odb.Core.Mock.MockStorageEngine()).GetMetaRepresentation(user, ci, true
				, null, new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.DefaultInstrospectionCallbackForStore
				(null, null, false));
			AssertTrue(comparator.HasChanged(instanceInfo, instanceInfo3));
			AssertEquals(1, comparator.GetNbChanges());
			AssertEquals(1, comparator.GetAttributeToSetToNull().Count);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.SetAttributeToNullAction o = (NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.SetAttributeToNullAction
				)comparator.GetAttributeToSetToNull()[0];
			AssertEquals(0, comparator.GetChangedAttributeActions().Count);
			AssertEquals(2, o.GetAttributeId());
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestCompareCollection9()
		{
			NeoDatis.Odb.Test.VO.Login.Function function = new NeoDatis.Odb.Test.VO.Login.Function
				("login");
			NeoDatis.Odb.Test.VO.Login.Profile profile = new NeoDatis.Odb.Test.VO.Login.Profile
				("operator", function);
			NeoDatis.Odb.Test.VO.Login.User user = new NeoDatis.Odb.Test.VO.Login.User("olivier smadja"
				, "olivier@neodatis.com", profile);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.IObjectInfoComparator comparator = new 
				NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Compare.ObjectInfoComparator();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci = classIntrospector.Introspect(
				user.GetType(), true).GetMainClassInfo();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo instanceInfo = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.LocalObjectIntrospector(new 
				NeoDatis.Odb.Core.Mock.MockStorageEngine()).GetMetaRepresentation(user, ci, true
				, null, new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.DefaultInstrospectionCallbackForStore
				(null, null, false));
			// Sets attributes offsets - this is normally done by reading then from
			// disk, but in this junit,
			// we must set them manually
			long[] offsets = new long[] { 1L, 2L, 3L };
			int[] ids = new int[] { 1, 2, 3 };
			instanceInfo.GetHeader().SetAttributesIdentification(offsets);
			instanceInfo.GetHeader().SetAttributesIds(ids);
			instanceInfo.GetHeader().SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(1
				));
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoiProfile = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)instanceInfo.GetAttributeValueFromId(2);
			nnoiProfile.SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(2));
			user.SetName("Kiko");
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo instanceInfo3 = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.LocalObjectIntrospector(new 
				NeoDatis.Odb.Core.Mock.MockStorageEngine()).GetMetaRepresentation(user, ci, true
				, null, new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.DefaultInstrospectionCallbackForStore
				(null, null, false));
			instanceInfo3.GetHeader().SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(
				1));
			nnoiProfile = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo)instanceInfo3
				.GetAttributeValueFromId(2);
			nnoiProfile.SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(2));
			AssertTrue(comparator.HasChanged(instanceInfo, instanceInfo3));
			AssertEquals(1, comparator.GetNbChanges());
			NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.ChangedNativeAttributeAction cnaa = 
				(NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.ChangedNativeAttributeAction)comparator
				.GetChangedAttributeActions()[0];
			AssertEquals(1, comparator.GetChangedAttributeActions().Count);
			AssertEquals(user.GetName(), cnaa.GetNoiWithNewValue().GetObject());
		}

		[Test]
        public virtual void TestGetSuperClasses()
		{
			System.Collections.IList superclasses = classIntrospector.GetSuperClasses(typeof(
				NeoDatis.Odb.Test.VO.Inheritance.FootballPlayer).FullName, true);
			AssertEquals(3, superclasses.Count);
			AssertEquals(typeof(NeoDatis.Odb.Test.VO.Inheritance.FootballPlayer), superclasses
				[0]);
			AssertEquals(typeof(NeoDatis.Odb.Test.VO.Inheritance.OutdoorPlayer), superclasses
				[1]);
			AssertEquals(typeof(NeoDatis.Odb.Test.VO.Inheritance.Player), superclasses[2]);
		}

		[Test]
        public virtual void TestGetAllFields()
		{
			System.Collections.IList allFields = classIntrospector.GetAllFields(typeof(NeoDatis.Odb.Test.VO.Inheritance.FootballPlayer
				).FullName);
			AssertEquals(3, allFields.Count);
			AssertEquals("role", ((System.Reflection.FieldInfo)allFields[0]).Name);
			AssertEquals("groundName", ((System.Reflection.FieldInfo)allFields[1]).Name);
			AssertEquals("name", ((System.Reflection.FieldInfo)allFields[2]).Name);
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestIntrospectWithNull()
		{
			NeoDatis.Odb.Test.VO.Login.User user = new NeoDatis.Odb.Test.VO.Login.User("olivier smadja"
				, "olivier@neodatis.com", null);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.IObjectInfoComparator comparator = new 
				NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Compare.ObjectInfoComparator();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci = classIntrospector.Introspect(
				user.GetType(), true).GetMainClassInfo();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo instanceInfo = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.LocalObjectIntrospector(new 
				NeoDatis.Odb.Core.Mock.MockStorageEngine()).GetMetaRepresentation(user, ci, true
				, null, new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.DefaultInstrospectionCallbackForStore
				(null, null, false));
			// Sets attributes offsets - this is normally done by reading then from
			// disk, but in this junit,
			// we must set them manually
			long[] offsets = new long[] { 1L, 2L, 3L };
			int[] ids = new int[] { 1, 2, 3 };
			instanceInfo.GetHeader().SetAttributesIdentification(offsets);
			instanceInfo.GetHeader().SetAttributesIds(ids);
			instanceInfo.GetHeader().SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(1
				));
			object o = instanceInfo.GetAttributeValueFromId(2);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoiProfile = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)o;
			nnoiProfile.SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(2));
			user.SetName("Olivier Smadja");
			user.SetEmail("olivier@neodatis.org");
			user.SetProfile(new NeoDatis.Odb.Test.VO.Login.Profile("pname"));
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo instanceInfo3 = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.LocalObjectIntrospector(new 
				NeoDatis.Odb.Core.Mock.MockStorageEngine()).GetMetaRepresentation(user, ci, true
				, null, new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.DefaultInstrospectionCallbackForStore
				(null, null, false));
			instanceInfo3.GetHeader().SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(
				1));
			nnoiProfile = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo)instanceInfo3
				.GetAttributeValueFromId(2);
			nnoiProfile.SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(2));
			AssertTrue(comparator.HasChanged(instanceInfo, instanceInfo3));
			AssertEquals(3, comparator.GetNbChanges());
			AssertEquals(2, comparator.GetChangedAttributeActions().Count);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.ChangedNativeAttributeAction cnaa = 
				(NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.ChangedNativeAttributeAction)comparator
				.GetChangedAttributeActions()[0];
			AssertEquals("Olivier Smadja", cnaa.GetNoiWithNewValue().GetObject());
			cnaa = (NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.ChangedNativeAttributeAction
				)comparator.GetChangedAttributeActions()[1];
			AssertEquals("olivier@neodatis.org", cnaa.GetNoiWithNewValue().GetObject());
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestIntrospectWithNull2()
		{
			NeoDatis.Odb.Test.VO.Login.User user = new NeoDatis.Odb.Test.VO.Login.User("olivier smadja"
				, "olivier@neodatis.com", null);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.IObjectInfoComparator comparator = new 
				NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Compare.ObjectInfoComparator();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci = classIntrospector.Introspect(
				user.GetType(), true).GetMainClassInfo();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo instanceInfo = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.LocalObjectIntrospector(new 
				NeoDatis.Odb.Core.Mock.MockStorageEngine()).GetMetaRepresentation(user, ci, true
				, null, new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.DefaultInstrospectionCallbackForStore
				(null, null, false));
			// Sets attributes offsets - this is normally done by reading then from
			// disk, but in this junit,
			// we must set them manually
			long[] offsets = new long[] { 1L, 2L, 3L };
			int[] ids = new int[] { 1, 2, 3 };
			instanceInfo.GetHeader().SetAttributesIdentification(offsets);
			instanceInfo.GetHeader().SetAttributesIds(ids);
			instanceInfo.GetHeader().SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(1
				));
			object o = instanceInfo.GetAttributeValueFromId(2);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoiProfile = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)o;
			nnoiProfile.SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(2));
			user.SetProfile(new NeoDatis.Odb.Test.VO.Login.Profile("pname"));
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo instanceInfo3 = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.LocalObjectIntrospector(new 
				NeoDatis.Odb.Core.Mock.MockStorageEngine()).GetMetaRepresentation(user, ci, true
				, null, new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.DefaultInstrospectionCallbackForStore
				(null, null, false));
			instanceInfo3.GetHeader().SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(
				1));
			nnoiProfile = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo)instanceInfo3
				.GetAttributeValueFromId(2);
			nnoiProfile.SetOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(2));
			bool b = comparator.HasChanged(instanceInfo, instanceInfo3);
			AssertTrue(b);
			AssertEquals(1, comparator.GetNbChanges());
			AssertEquals(0, comparator.GetChangedAttributeActions().Count);
			AssertEquals(1, comparator.GetNewObjectMetaRepresentations().Count);
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestGetDependentObjects()
		{
			NeoDatis.Odb.Test.VO.Login.User user = new NeoDatis.Odb.Test.VO.Login.User("olivier smadja"
				, "olivier@neodatis.com", new NeoDatis.Odb.Test.VO.Login.Profile("operator", new 
				NeoDatis.Odb.Test.VO.Login.Function("login")));
			NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.GetDependentObjectIntrospectingCallback
				 callback = new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.GetDependentObjectIntrospectingCallback
				();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci = classIntrospector.Introspect(
				user.GetType(), true).GetMainClassInfo();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo instanceInfo = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.LocalObjectIntrospector(new 
				NeoDatis.Odb.Core.Mock.MockStorageEngine()).GetMetaRepresentation(user, ci, true
				, null, callback);
			AssertEquals(user.GetType().FullName, instanceInfo.GetClassInfo().GetFullClassName
				());
			AssertEquals("olivier smadja", instanceInfo.GetAttributeValueFromId(ci.GetAttributeId
				("name")).ToString());
			AssertEquals(typeof(NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo), 
				instanceInfo.GetAttributeValueFromId(ci.GetAttributeId("name")).GetType());
			System.Collections.ICollection objects = callback.GetObjects();
			AssertEquals(2, objects.Count);
			AssertTrue(objects.Contains(user.GetProfile()));
			AssertTrue(objects.Contains(user.GetProfile().GetFunctions()[0]));
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestCopy()
		{
			NeoDatis.Odb.Test.VO.Login.Function function = new NeoDatis.Odb.Test.VO.Login.Function
				("login");
			NeoDatis.Odb.Test.VO.Login.Profile profile = new NeoDatis.Odb.Test.VO.Login.Profile
				("operator", function);
			NeoDatis.Odb.Test.VO.Login.User user = new NeoDatis.Odb.Test.VO.Login.User("olivier smadja"
				, "olivier@neodatis.org", profile);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci = classIntrospector.Introspect(
				user.GetType(), true).GetMainClassInfo();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo instanceInfo = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.LocalObjectIntrospector(new 
				NeoDatis.Odb.Core.Mock.MockStorageEngine()).GetMetaRepresentation(user, ci, true
				, null, new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.DefaultInstrospectionCallbackForStore
				(null, null, false));
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo copy = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)instanceInfo.CreateCopy(new NeoDatis.Tool.Wrappers.Map.OdbHashMap(), true);
			AssertEquals(3, copy.GetAttributeValues().Length);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo[] aois = copy.GetAttributeValues
				();
			for (int i = 0; i < aois.Length; i++)
			{
				NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo aoi = aois[i];
				AssertEquals(instanceInfo.GetAttributeValues()[i].GetOdbTypeId(), aoi.GetOdbTypeId
					());
			}
		}
	}
}
