namespace NeoDatis.Odb.Test.Oid
{
	public class TestOid : NeoDatis.Odb.Test.ODBTest
	{
		public virtual void TestEquals()
		{
			NeoDatis.Odb.OID oid1 = NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(1);
			NeoDatis.Odb.OID oid2 = NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(1);
			AssertEquals(oid1, oid2);
		}

		public virtual void TestOIdInMap()
		{
			NeoDatis.Odb.OID oid1 = NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(1);
			NeoDatis.Odb.OID oid2 = NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(1);
			System.Collections.IDictionary map = new NeoDatis.Tool.Wrappers.Map.OdbHashMap();
			map.Add(oid1, "oid1");
			AssertNotNull(map[oid2]);
		}

		/// <summary>Performance test.</summary>
		/// <remarks>Performance test. Using ID or long in hash map</remarks>
		public virtual void TestPerformanceLong()
		{
			int size = 300000;
			System.Collections.IDictionary longMap = new NeoDatis.Tool.Wrappers.Map.OdbHashMap
				();
			// LONG
			NeoDatis.Tool.StopWatch timeLongMapCreation = new NeoDatis.Tool.StopWatch();
			timeLongMapCreation.Start();
			// Creates a hashmap with 100000 Longs
			for (int i = 0; i < size; i++)
			{
				longMap.Add(System.Convert.ToInt64(i), i.ToString());
			}
			timeLongMapCreation.End();
			NeoDatis.Tool.StopWatch timeLongMapGet = new NeoDatis.Tool.StopWatch();
			timeLongMapGet.Start();
			// get all map elements
			for (int i = 0; i < size; i++)
			{
				AssertNotNull(longMap[System.Convert.ToInt64(i)]);
			}
			timeLongMapGet.End();
			Println(size + " objects : Long Map creation=" + timeLongMapCreation.GetDurationInMiliseconds
				() + " - get=" + timeLongMapGet.GetDurationInMiliseconds());
		}

		/// <summary>Performance test.</summary>
		/// <remarks>Performance test. Using ID or long in hash map</remarks>
		public virtual void TestPerformanceOid()
		{
			int size = 300000;
			System.Collections.IDictionary oidMap = new NeoDatis.Tool.Wrappers.Map.OdbHashMap
				();
			// OID
			NeoDatis.Tool.StopWatch timeOidMapCreation = new NeoDatis.Tool.StopWatch();
			timeOidMapCreation.Start();
			// Creates a hashmap with 100000 Longs
			for (int i = 0; i < size; i++)
			{
				oidMap.Add(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(i), i.ToString());
			}
			timeOidMapCreation.End();
			NeoDatis.Tool.StopWatch timeOidMapGet = new NeoDatis.Tool.StopWatch();
			timeOidMapGet.Start();
			// get all map elements
			for (int i = 0; i < size; i++)
			{
				AssertNotNull(oidMap[NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(i)]);
			}
			timeOidMapGet.End();
			Println(size + " objects : OID Map creation=" + timeOidMapCreation.GetDurationInMiliseconds
				() + " - get=" + timeOidMapGet.GetDurationInMiliseconds());
		}

		public virtual void TestAndy1()
		{
			NeoDatis.Odb.ODB odb = Open("test-oid");
			NeoDatis.Odb.Test.Oid.B b1 = new NeoDatis.Odb.Test.Oid.B("b");
			NeoDatis.Odb.Test.Oid.A a1 = new NeoDatis.Odb.Test.Oid.A("a", b1);
			odb.Store(a1);
			NeoDatis.Odb.OID oida = odb.GetObjectId(a1);
			NeoDatis.Odb.OID oidb = odb.GetObjectId(b1);
			odb.Close();
			odb = Open("test-oid");
			NeoDatis.Odb.Test.Oid.A a2 = (NeoDatis.Odb.Test.Oid.A)odb.GetObjectFromId(oida);
			NeoDatis.Odb.Test.Oid.B b2 = (NeoDatis.Odb.Test.Oid.B)odb.GetObjectFromId(oidb);
			odb.Close();
			AssertNotNull(a2);
			AssertNotNull(b2);
			AssertNotNull(a2.GetB());
		}

		public virtual void TestAndy2()
		{
			// LogUtil.allOn(true);
			NeoDatis.Odb.ODB odb = Open("test-oid");
			NeoDatis.Odb.Test.Oid.B b1 = new NeoDatis.Odb.Test.Oid.B("b");
			NeoDatis.Odb.Test.Oid.A a1 = new NeoDatis.Odb.Test.Oid.A("a", b1);
			odb.Store(a1);
			long oida = ((NeoDatis.Odb.OID)odb.GetObjectId(a1)).GetObjectId();
			long oidb = ((NeoDatis.Odb.OID)odb.GetObjectId(b1)).GetObjectId();
			odb.Close();
			odb = Open("test-oid");
			NeoDatis.Odb.Test.Oid.A a2 = (NeoDatis.Odb.Test.Oid.A)odb.GetObjectFromId(new NeoDatis.Odb.Impl.Core.Oid.OdbObjectOID
				(oida));
			NeoDatis.Odb.Test.Oid.B b2 = (NeoDatis.Odb.Test.Oid.B)odb.GetObjectFromId(new NeoDatis.Odb.Impl.Core.Oid.OdbObjectOID
				(oidb));
			odb.Close();
			AssertNotNull(a2);
			AssertNotNull(b2);
			AssertNotNull(a2.GetB());
		}

		public virtual void TestAndy3()
		{
			NeoDatis.Odb.ODB odb = Open("test-oid");
			try
			{
				NeoDatis.Odb.Test.Oid.A a2 = (NeoDatis.Odb.Test.Oid.A)odb.GetObjectFromId(new NeoDatis.Odb.Impl.Core.Oid.OdbObjectOID
					(34));
				Fail("Should have thrown Exception");
			}
			catch (System.Exception)
			{
			}
		}
		// ok must enter the catch block
	}
}
