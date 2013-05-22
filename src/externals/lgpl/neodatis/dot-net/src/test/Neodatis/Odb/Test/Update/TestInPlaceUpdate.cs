using NeoDatis.Odb.Test.VO.Login;
using NeoDatis.Odb.Core.Query.Criteria;
using NeoDatis.Odb.Impl.Core.Query.Criteria;
using NUnit.Framework;
namespace NeoDatis.Odb.Test.Update
{
	[TestFixture]
    public class TestInPlaceUpdate : NeoDatis.Odb.Test.ODBTest
	{
		public static readonly string Name = "in-place.neodatis";

		public static readonly string Name2 = "in-place-no.neodatis";

		public const int Size = 50;

		/// <summary>Stores an object User that has a non null reference to a Profile.</summary>
		/// <remarks>
		/// Stores an object User that has a non null reference to a Profile. Then
		/// deletes the profile. Loads the user again and updates the user profile
		/// with a new created profile. ODB did not detect the change Detected by
		/// Olivier.
		/// </remarks>
		/// <exception cref="System.Exception">System.Exception</exception>
		[Test]
        public virtual void Test8()
		{
			// reset counter to checks update type (normal or updates)
			NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.AbstractObjectWriter.ResetNbUpdates();
			if (!isLocal)
			{
				return;
			}
			DeleteBase(Name);
			NeoDatis.Odb.ODB odb = Open(Name);
			User user = new User("name"
				, "email", new Profile("p1", new Function
				("function")));
			odb.Store(user);
			odb.Close();
			odb = Open(Name);
			Profile p = (Profile)odb.GetObjects<Profile>().GetFirst();
			odb.Delete(p);
			odb.Close();
			odb = Open(Name);
			User user3 = (User)odb.GetObjects<User>().GetFirst();
			AssertNull(user3.GetProfile());
			user3.SetProfile(new Profile("new profile", new Function
				("f1")));
			user3.SetEmail("email2");
			user3.SetName("name2");
			odb.Store(user3);
			odb.Close();
			odb = Open(Name);
			User user4 = (User)odb.GetObjects<User>().GetFirst();
			odb.Close();
			DeleteBase(Name);
			AssertEquals("new profile", user4.GetProfile().GetName());
			AssertEquals("email2", user4.GetEmail());
			AssertEquals("name2", user4.GetName());
		}

		/// <summary>Stores an object User that has a non null reference to a Profile.</summary>
		/// <remarks>
		/// Stores an object User that has a non null reference to a Profile. Creates
		/// a new profile.
		/// Update the last profile and sets it a the new user profile.ODB detects
		/// the reference change but does not update the profile Detected by Olivier.
		/// 22/05/2007
		/// </remarks>
		/// <exception cref="System.Exception">System.Exception</exception>
		[Test]
        public virtual void Test9()
		{
			// reset counter to checks update type (normal or updates)
			NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.AbstractObjectWriter.ResetNbUpdates();
			DeleteBase(Name);
			NeoDatis.Odb.ODB odb = Open(Name);
			User user = new User("name"
				, "email", new Profile("p1", new Function
				("function")));
			odb.Store(user);
			odb.Store(new Profile("new profile"));
			odb.Close();
			odb = Open(Name);
			Profile p = (Profile)odb.GetObjects<Profile>(new CriteriaQuery(Where.Equal("name", "new profile"))).GetFirst
				();
			p.SetName("new profile2");
			User user2 = (User)odb.GetObjects<User>().GetFirst();
			user2.SetProfile(p);
			odb.Store(user2);
			odb.Close();
			odb = Open(Name);
			User user3 = (User)odb.GetObjects<User>().GetFirst();
			AssertNotNull(user3.GetProfile());
			odb.Close();
			DeleteBase(Name);
			AssertEquals("new profile2", user3.GetProfile().GetName());
		}

		/// <summary>test in place update with rollback.</summary>
		/// <remarks>
		/// test in place update with rollback. Bug detected by Olivier 22/02/2008.
		/// In place updates for connected object were done out of transaction,
		/// avoiding rollback (ObejctWriter.manageInPlaceUpdate()
		/// </remarks>
		[Test]
        public virtual void Test10()
		{
			NeoDatis.Odb.ODB odb = null;
			try
			{
				odb = Open("inplcae-transaction");
				NeoDatis.Odb.OID oid = odb.Store(new Function("function1"
					));
				odb.Close();
				odb = Open("inplcae-transaction");
				Function f = (Function)odb.
					GetObjectFromId(oid);
				f.SetName("function2");
				odb.Store(f);
				odb.Rollback();
				odb.Close();
				odb = Open("inplcae-transaction");
				f = (Function)odb.GetObjectFromId(oid);
				odb.Close();
				AssertEquals("function1", f.GetName());
			}
			catch (System.Exception)
			{
			}
		}
		// TODO: handle exception
	}
}
