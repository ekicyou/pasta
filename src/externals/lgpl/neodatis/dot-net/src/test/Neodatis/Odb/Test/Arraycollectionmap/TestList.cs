using NeoDatis.Odb.Test.VO.Arraycollectionmap;
using NeoDatis.Odb.Impl.Core.Query.Criteria;
using NeoDatis.Odb.Core.Query.Criteria;
using NeoDatis.Odb.Test.VO.Sport;

using NUnit.Framework;
using System;
namespace NeoDatis.Odb.Test.Arraycollectionmap
{
	[TestFixture]
    public class TestList : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestList1()
		{
			DeleteBase("list1.neodatis");
			NeoDatis.Odb.ODB odb = Open("list1.neodatis");
			long nb = odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery(typeof(
				PlayerWithList)));
			PlayerWithList player = new PlayerWithList
				("kiko");
			player.AddGame("volley-ball");
			player.AddGame("squash");
			player.AddGame("tennis");
			player.AddGame("ping-pong");
			odb.Store(player);
			odb.Close();
			NeoDatis.Odb.ODB odb2 = Open("list1.neodatis");
			NeoDatis.Odb.Objects<PlayerWithList> l = odb2.GetObjects<PlayerWithList>(true);
			Println(l);
			AssertEquals(nb + 1, l.Count);
			// gets last player
			PlayerWithList player2 = (PlayerWithList)l.GetFirst();
			AssertEquals(player.ToString(), player2.ToString());
			odb2.Close();
			DeleteBase("list1.neodatis");
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestList1WithNull()
		{
			DeleteBase("list1.neodatis");
			NeoDatis.Odb.ODB odb = Open("list1.neodatis");
			long nb = odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery(typeof(
				PlayerWithList)));
			PlayerWithList player = new PlayerWithList("kiko");
			player.AddGame("volley-ball");
			player.AddGame("squash");
			player.AddGame("tennis");
			player.AddGame(null);
			odb.Store(player);
			odb.Close();
			NeoDatis.Odb.ODB odb2 = Open("list1.neodatis");
			NeoDatis.Odb.Objects<PlayerWithList> l = odb2.GetObjects<PlayerWithList>(true);
			AssertEquals(nb + 1, l.Count);
			// gets last player
			PlayerWithList player2 = (PlayerWithList)l.GetFirst();
			AssertEquals(player.GetGame(2), player2.GetGame(2));
			odb2.Close();
			DeleteBase("list1.neodatis");
		}
        [Test]
        public virtual void TestBigList()
        {
            string baseName = GetBaseName();
            NeoDatis.Odb.ODB odb = Open(baseName);
            int size = 10000;
            int size2 = 4;
            long t0 = DateTime.Now.Ticks;
            for (int i = 0; i < size; i++)
            {
                PlayerWithList player = new PlayerWithList("player " + i);
                for (int j = 0; j < size2; j++)
                {
                    player.AddGame("game " + j);
                }
                odb.Store(player);
            }
            odb.Close();
            long t1 = DateTime.Now.Ticks;
            Console.WriteLine("insert : " + (t1 - t0) / 10000);

            NeoDatis.Odb.ODB odb2 = Open(baseName);
            NeoDatis.Odb.Objects<PlayerWithList> l = odb2.GetObjects<PlayerWithList>(false);
            long t2 = DateTime.Now.Ticks;
            AssertEquals(size, l.Count);
            Console.WriteLine("get objects " +l.Count + " : " + (t2 - t1) / 10000);
            odb2.Close();
            DeleteBase(baseName);
        }


		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestList2()
		{
			DeleteBase("list1.neodatis");
			NeoDatis.Odb.ODB odb = Open("list1.neodatis");
			long nb = odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery(typeof(
				PlayerWithList)));
			PlayerWithList player = new PlayerWithList
				("kiko");
			player.SetGames(null);
			odb.Store(player);
			odb.Close();
			NeoDatis.Odb.ODB odb2 = Open("list1.neodatis");
			NeoDatis.Odb.Objects<PlayerWithList> l = odb2.GetObjects<PlayerWithList>(true);
			AssertEquals(nb + 1, l.Count);
			odb2.Close();
			DeleteBase("list1.neodatis");
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestList3()
		{
			DeleteBase("list3.neodatis");
			NeoDatis.Odb.ODB odb = Open("list3.neodatis");
			long nb = odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery(typeof(
				MyObject)));
			MyList l1 = new MyList
				();
			l1.Add("object1");
			l1.Add("object2");
			MyObject myObject = new MyObject("o1", l1);
			odb.Store(myObject);
			odb.Close();
			NeoDatis.Odb.ODB odb2 = Open("list3.neodatis");
			Objects<MyObject> l = odb2.GetObjects<MyObject>(true);
			AssertEquals(nb + 1, l.Count);
			odb2.Close();
			DeleteBase("list3.neodatis");
		}

		/// <summary>Test update object list.</summary>
		/// <remarks>Test update object list. Removing one, adding other</remarks>
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestList4Update()
		{
			DeleteBase("list4.neodatis");
			NeoDatis.Odb.ODB odb = Open("list4.neodatis");
			long nb = odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery(typeof(
				MyObject)));
			MyList l1 = new MyList
				();
			l1.Add("object1");
			l1.Add("object2");
			MyObject myObject = new MyObject
				("o1", l1);
			odb.Store(myObject);
			odb.Close();
			NeoDatis.Odb.ODB odb2 = Open("list4.neodatis");
			NeoDatis.Odb.Objects<MyObject> l = odb2.GetObjects<MyObject>(true);
			MyObject mo = (MyObject
				)l.GetFirst();
			mo.GetList().RemoveAt(1);
			mo.GetList().Add("object 2bis");
			odb2.Store(mo);
			odb2.Close();
			odb2 = Open("list4.neodatis");
			l = odb2.GetObjects<MyObject>(true);
			AssertEquals(nb + 1, l.Count);
			MyObject mo2 = (MyObject
				)l.GetFirst();
			AssertEquals("object1", mo2.GetList()[0]);
			AssertEquals("object 2bis", mo2.GetList()[1]);
			odb2.Close();
			DeleteBase("list4.neodatis");
		}

		/// <summary>Test update object list.</summary>
		/// <remarks>Test update object list. adding 2 elements</remarks>
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestList4Update2()
		{
			DeleteBase("list4.neodatis");
			NeoDatis.Odb.ODB odb = Open("list4.neodatis");
			long nb = odb.Count(new CriteriaQuery(typeof(
				MyObject)));
			MyList l1 = new MyList
				();
			l1.Add("object1");
			l1.Add("object2");
			MyObject myObject = new MyObject("o1", l1);
			odb.Store(myObject);
			odb.Close();
			NeoDatis.Odb.ODB odb2 = Open("list4.neodatis");
			NeoDatis.Odb.Objects<MyObject> l = odb2.GetObjects<MyObject>(true);
			MyObject mo = (MyObject
				)l.GetFirst();
			mo.GetList().Add("object3");
			mo.GetList().Add("object4");
			odb2.Store(mo);
			odb2.Close();
			odb2 = Open("list4.neodatis");
			l = odb2.GetObjects<MyObject>(true);
			AssertEquals(nb + 1, l.Count);
			MyObject mo2 = l.GetFirst();
			AssertEquals(4, mo2.GetList().Count);
			AssertEquals("object1", mo2.GetList()[0]);
			AssertEquals("object2", mo2.GetList()[1]);
			AssertEquals("object3", mo2.GetList()[2]);
			AssertEquals("object4", mo2.GetList()[3]);
			odb2.Close();
			DeleteBase("list4.neodatis");
		}

		/// <summary>Test update object list.</summary>
		/// <remarks>Test update object list. A list of Integer</remarks>
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestList4Update3()
		{
			DeleteBase("list5.neodatis");
			NeoDatis.Odb.ODB odb = Open("list5.neodatis");
			ObjectWithListOfInteger o = new ObjectWithListOfInteger
				("test");
			o.GetListOfIntegers().Add(System.Convert.ToInt32("100"));
			odb.Store(o);
			odb.Close();
			NeoDatis.Odb.ODB odb2 = Open("list5.neodatis");
			NeoDatis.Odb.Objects<ObjectWithListOfInteger> l = odb2.GetObjects<ObjectWithListOfInteger>(true);
			ObjectWithListOfInteger o2 = (ObjectWithListOfInteger
				)l.GetFirst();
			o2.GetListOfIntegers().Clear();
			o2.GetListOfIntegers().Add(System.Convert.ToInt32("200"));
			odb2.Store(o2);
			odb2.Close();
			odb2 = Open("list5.neodatis");
			l = odb2.GetObjects<ObjectWithListOfInteger>(true);
			AssertEquals(1, l.Count);
			ObjectWithListOfInteger o3 = l.GetFirst();
			AssertEquals(1, o3.GetListOfIntegers().Count);
			AssertEquals(System.Convert.ToInt32("200"), o3.GetListOfIntegers()[0]);
			odb2.Close();
			DeleteBase("list5.neodatis");
		}

		/// <summary>Test update object list.</summary>
		/// <remarks>Test update object list. A list of Integer. 1000 updates</remarks>
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestList4Update4()
		{
			DeleteBase("list5.neodatis");
			NeoDatis.Odb.ODB odb = Open("list5.neodatis");
			ObjectWithListOfInteger o = new ObjectWithListOfInteger
				("test");
			o.GetListOfIntegers().Add(System.Convert.ToInt32("100"));
			odb.Store(o);
			odb.Close();
			int size = isLocal ? 1000 : 100;
			for (int i = 0; i < size; i++)
			{
				NeoDatis.Odb.ODB odb2 = Open("list5.neodatis");
				NeoDatis.Odb.Objects<ObjectWithListOfInteger> ll = odb2.GetObjects<ObjectWithListOfInteger>(true);
				ObjectWithListOfInteger o2 = (ObjectWithListOfInteger)ll.GetFirst();
				o2.GetListOfIntegers().Clear();
				o2.GetListOfIntegers().Add(200 + i);
				odb2.Store(o2);
				odb2.Close();
			}
			NeoDatis.Odb.ODB odb3 = Open("list5.neodatis");
			NeoDatis.Odb.Objects<ObjectWithListOfInteger> l = odb3.GetObjects<ObjectWithListOfInteger>(true);
			AssertEquals(1, l.Count);
			ObjectWithListOfInteger o3 = (ObjectWithListOfInteger)l.GetFirst();
			AssertEquals(1, o3.GetListOfIntegers().Count);
			AssertEquals(200 + size - 1, o3.GetListOfIntegers()[0]);
			odb3.Close();
			DeleteBase("list5.neodatis");
		}

		/// <summary>Test update object list.</summary>
		/// <remarks>
		/// Test update object list. A list of Integer. 1000 updates of an object
		/// that is the middle of the list
		/// </remarks>
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestList4Update4Middle()
		{
			DeleteBase("list5.neodatis");
			NeoDatis.Odb.ODB odb = Open("list5.neodatis");
			ObjectWithListOfInteger o = new ObjectWithListOfInteger
				("test1");
			o.GetListOfIntegers().Add(System.Convert.ToInt32("101"));
			odb.Store(o);
			o = new ObjectWithListOfInteger("test2");
			o.GetListOfIntegers().Add(System.Convert.ToInt32("102"));
			odb.Store(o);
			o = new ObjectWithListOfInteger("test3");
			o.GetListOfIntegers().Add(System.Convert.ToInt32("103"));
			odb.Store(o);
			odb.Close();
			int size = isLocal ? 1000 : 100;
			for (int i = 0; i < size; i++)
			{
				NeoDatis.Odb.ODB odb2 = Open("list5.neodatis");
				NeoDatis.Odb.Objects<ObjectWithListOfInteger> ll = odb2.GetObjects<ObjectWithListOfInteger>(new CriteriaQuery(Where.Equal("name", "test2")));
				ObjectWithListOfInteger o2 = ll.GetFirst();
				o2.GetListOfIntegers().Clear();
				o2.GetListOfIntegers().Add(200 + i);
				odb2.Store(o2);
				odb2.Close();
			}
			NeoDatis.Odb.ODB odb3 = Open("list5.neodatis");
			NeoDatis.Odb.Objects<ObjectWithListOfInteger> l = odb3.GetObjects<ObjectWithListOfInteger>(new CriteriaQuery( Where.Equal("name", "test2")));
			AssertEquals(1, l.Count);
			ObjectWithListOfInteger o3 = l.GetFirst();
			AssertEquals(1, o3.GetListOfIntegers().Count);
			AssertEquals(200 + size - 1, o3.GetListOfIntegers()[0]);
			odb3.Close();
			DeleteBase("list5.neodatis");
		}

		/// <summary>Test update object list.</summary>
		/// <remarks>
		/// Test update object list. A list of Integer. 1000 updates, increasing
		/// number of elements
		/// </remarks>
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestList4Update5()
		{
			DeleteBase("list5.neodatis");
			NeoDatis.Odb.ODB odb = Open("list5.neodatis");
			ObjectWithListOfInteger o = new ObjectWithListOfInteger
				("test");
			o.GetListOfIntegers().Add(System.Convert.ToInt32("100"));
			odb.Store(o);
			odb.Close();
			int size = isLocal ? 1000 : 100;
			for (int i = 0; i < size; i++)
			{
				NeoDatis.Odb.ODB odb2 = Open("list5.neodatis");
				NeoDatis.Odb.Objects<ObjectWithListOfInteger> ll = odb2.GetObjects<ObjectWithListOfInteger>(true);
				ObjectWithListOfInteger o2 = (ObjectWithListOfInteger
					)ll.GetFirst();
				o2.GetListOfIntegers().Add(200 + i);
				odb2.Store(o2);
				odb2.Close();
			}
			NeoDatis.Odb.ODB odb3 = Open("list5.neodatis");
			NeoDatis.Odb.Objects<ObjectWithListOfInteger> l = odb3.GetObjects<ObjectWithListOfInteger>(true);
			AssertEquals(1, l.Count);
			ObjectWithListOfInteger o3 = l.GetFirst();
			AssertEquals(size + 1, o3.GetListOfIntegers().Count);
			odb3.Close();
			DeleteBase("list5.neodatis");
		}

		/// <summary>Test update object list.</summary>
		/// <remarks>
		/// Test update object list. A list of Integer. 1000 updates of an object
		/// increasing list nb elements that is the middle of the list
		/// </remarks>
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestList4Update4Middle2()
		{
			DeleteBase("list5.neodatis");
			NeoDatis.Odb.ODB odb = Open("list5.neodatis");
			ObjectWithListOfInteger o = new ObjectWithListOfInteger
				("test1");
			o.GetListOfIntegers().Add(System.Convert.ToInt32("101"));
			odb.Store(o);
			o = new ObjectWithListOfInteger("test2");
			o.GetListOfIntegers().Add(System.Convert.ToInt32("102"));
			odb.Store(o);
			o = new ObjectWithListOfInteger("test3");
			o.GetListOfIntegers().Add(System.Convert.ToInt32("103"));
			odb.Store(o);
			odb.Close();
			int size = isLocal ? 1000 : 100;
			for (int i = 0; i < size; i++)
			{
				NeoDatis.Odb.ODB odb2 = Open("list5.neodatis");
				NeoDatis.Odb.Objects<ObjectWithListOfInteger> ll = odb2.GetObjects<ObjectWithListOfInteger>(new CriteriaQuery(Where.Equal("name", "test2")));
				ObjectWithListOfInteger o2 = ll.GetFirst();
				o2.GetListOfIntegers().Add(200 + i);
				odb2.Store(o2);
				odb2.Close();
			}
			NeoDatis.Odb.ODB odb3 = Open("list5.neodatis");
			NeoDatis.Odb.Objects<ObjectWithListOfInteger> l = odb3.GetObjects<ObjectWithListOfInteger>(new CriteriaQuery(Where.Equal("name", "test2")));
			AssertEquals(1, l.Count);
			ObjectWithListOfInteger o3 = l.GetFirst();
			AssertEquals(1 + size, o3.GetListOfIntegers().Count);
			odb3.Close();
			DeleteBase("list5.neodatis");
		}

		/// <summary>one object has a list.</summary>
		/// <remarks>
		/// one object has a list. we delete one of the object of the list of the
		/// object. And the main object still has it
		/// </remarks>
		/// <exception cref="System.Exception">System.Exception</exception>
		[Test]
        public virtual void TestDeletingOneElementOfTheList()
		{
			if (!testNewFeature)
			{
				return;
			}
			string baseName = GetBaseName();
			DeleteBase(baseName);
			NeoDatis.Odb.ODB odb = Open(baseName);
			NeoDatis.Odb.Test.VO.Sport.Team t1 = new NeoDatis.Odb.Test.VO.Sport.Team("team1");
			t1.AddPlayer(new Player("player1", new System.DateTime(), new Sport("sport1")));
			t1.AddPlayer(new Player("player2", new System.DateTime(), new Sport("sport2")));
			odb.Store(t1);
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Objects<Team> teams = odb.GetObjects<Team>();
			Team team = teams.GetFirst();
			AssertEquals(2, team.GetPlayers().Count);
			NeoDatis.Odb.Objects<Player> players = odb.GetObjects<Player>();
            Player p1 = players.GetFirst();
			odb.Delete(p1);
			odb.Close();
			AssertEquals(1, team.GetPlayers().Count);
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestCollectionWithContain()
		{
			NeoDatis.Odb.ODB odb = null;
			string baseName = GetBaseName();
			try
			{
				odb = Open(baseName);
				long nb = odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery(typeof(
					PlayerWithList)));
				PlayerWithList player = new PlayerWithList
					("kiko");
				player.AddGame("volley-ball");
				player.AddGame("squash");
				player.AddGame("tennis");
				player.AddGame("ping-pong");
				odb.Store(player);
				odb.Close();
				odb = Open(baseName);
				NeoDatis.Odb.Objects<PlayerWithList> l = odb.GetObjects<PlayerWithList>(new CriteriaQuery(Where.Contain("games", "tennis")));
				AssertEquals(nb + 1, l.Count);
			}
			catch (System.Exception e)
			{
				if (odb != null)
				{
					odb.Rollback();
					odb = null;
				}
				throw;
			}
			finally
			{
				if (odb != null)
				{
					odb.Close();
				}
			}
		}
	}
}
