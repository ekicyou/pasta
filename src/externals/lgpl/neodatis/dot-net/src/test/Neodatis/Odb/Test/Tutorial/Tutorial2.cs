using NUnit.Framework;
namespace NeoDatis.Odb.Test.Tutorial
{
	public class Tutorial2 : NeoDatis.Odb.Test.ODBTest
	{
		public static readonly string OdbName = "tutorial2.neodatis";

		/// <exception cref="System.Exception"></exception>
		public Tutorial2()
		{
			DeleteBase(OdbName);
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Step20()
		{
			// Create instance
			NeoDatis.Odb.Test.Tutorial.Sport sport = new NeoDatis.Odb.Test.Tutorial.Sport("volley-ball"
				);
			NeoDatis.Odb.ODB odb = null;
			NeoDatis.Odb.ODBServer server = null;
			try
			{
				// Creates the server on port 8000
				server = OpenServer(8000);
				// Tells the server to manage base 'base1' that points to the file
				// tutorial2.odb
				server.AddBase("base1", OdbName);
				// Then starts the server to run in background
				server.StartServer(true);
				// Open the databse client on the localhost on port 8000 and specify
				// which database instance
				odb = OpenClient("localhost", 8000, "base1");
				// Store the object
				odb.Store(sport);
			}
			finally
			{
				if (odb != null)
				{
					// First close the client
					odb.Close();
				}
				if (server != null)
				{
					// Then close the database server
					server.Close();
				}
			}
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test1()
		{
			NeoDatis.Odb.Test.Tutorial.Tutorial2 tutorial2 = new NeoDatis.Odb.Test.Tutorial.Tutorial2
				();
			tutorial2.Step20();
			//tutorial2.DisplayObjectsOf(typeof(NeoDatis.Odb.Test.Tutorial.Sport), "Step 20", " sport(s):"		);
		}

		/// <exception cref="System.Exception"></exception>
		public static void Main2(string[] args)
		{
			NeoDatis.Odb.Test.Tutorial.Tutorial2 tutorial2 = new NeoDatis.Odb.Test.Tutorial.Tutorial2
				();
			tutorial2.Step20();
			//tutorial2.DisplayObjectsOf(typeof(NeoDatis.Odb.Test.Tutorial.Sport), "Step 20", " sport(s):"	);
		}
	}
}
