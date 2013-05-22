using NUnit.Framework;
namespace NeoDatis.Odb.Test.Query.Values
{
	[System.Serializable]
	[TestFixture]
    public class TestCustomQueryFieldAction2 : NeoDatis.Odb.Impl.Core.Query.Values.CustomQueryFieldAction
	{
		/// <summary>The number of logins</summary>
		private long nbLoggedUsers;

		public TestCustomQueryFieldAction2()
		{
			this.nbLoggedUsers = 0;
		}

		/// <summary>The method that actually computes the logins</summary>
		public override void Execute(NeoDatis.Odb.OID oid, NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap
			 values)
		{
			// Gets the name of the user
			string userName = (string)values["name"];
			// Call an external class (Users) to check if the user is logged in
			if (NeoDatis.Odb.Test.Query.Values.Sessions.IsLogged(userName))
			{
				nbLoggedUsers++;
			}
		}

		public override object GetValue()
		{
			return nbLoggedUsers;
		}

		public override bool IsMultiRow()
		{
			return false;
		}

		public override void Start()
		{
		}

		// Nothing to do
		public override void End()
		{
		}

		// Nothing to do
		/// <exception cref="System.Exception"></exception>
		public static void Main2(string[] args)
		{
			NeoDatis.Odb.ODB odb = null;
			NeoDatis.Odb.Impl.Core.Query.Values.CustomQueryFieldAction customAction = new NeoDatis.Odb.Test.Query.Values.TestCustomQueryFieldAction
				();
			NeoDatis.Odb.Values values = odb.GetValues(new NeoDatis.Odb.Impl.Core.Query.Values.ValuesCriteriaQuery
				(typeof(NeoDatis.Odb.Test.Query.Values.Users)).Custom("nbLogins", "nb logged users"
				, customAction).Field("name"));
		}
	}

	internal class Sessions
	{
		public static bool IsLogged(string userName)
		{
			// TODO Auto-generated method stub
			return false;
		}
	}

	internal class Users
	{
	}
}
