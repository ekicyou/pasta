using NeoDatis.Odb.Test.VO.Login;
using NeoDatis.Odb.Test.VO.Sport;
using NUnit.Framework;
namespace NeoDatis.Odb.Test.Transaction
{
	[TestFixture]
    public class TestInTransaction : NeoDatis.Odb.Test.ODBTest
	{
		public readonly string BaseName = "transaction";

		/// <summary>Test select objects that are not yet commited</summary>
		/// <exception cref="System.Exception">System.Exception</exception>
		[Test]
        public virtual void TestSelectUnCommitedObject()
		{
			NeoDatis.Odb.ODB odb = null;
			try
			{
				DeleteBase(BaseName);
				odb = Open(BaseName);
				for (int i = 0; i < 4; i++)
				{
					odb.Store(new NeoDatis.Odb.Test.VO.Login.Function("function " + i));
				}
				odb.Close();
				// reopen the database
				odb = Open(BaseName);
				// stores a new function
				odb.Store(new NeoDatis.Odb.Test.VO.Login.Function("function uncommited"));
				NeoDatis.Odb.Objects<Function> functions = odb.GetObjects<Function>();
				AssertEquals(5, functions.Count);
			}
			finally
			{
				if (odb != null)
				{
					odb.Close();
					DeleteBase(BaseName);
				}
			}
		}

		/// <summary>Test select objects that are not yet commited</summary>
		/// <exception cref="System.Exception">System.Exception</exception>
		[Test]
        public virtual void TestSelectUnCommitedObject2()
		{
			NeoDatis.Odb.ODB odb = null;
			try
			{
				DeleteBase(BaseName);
				odb = Open(BaseName);
				for (int i = 0; i < 4; i++)
				{
					odb.Store(new NeoDatis.Odb.Test.VO.Login.User("user" + i, "email" + i, new NeoDatis.Odb.Test.VO.Login.Profile
						("profile" + i, new NeoDatis.Odb.Test.VO.Login.Function("function" + i))));
				}
				odb.Close();
				// reopen the database
				odb = Open(BaseName);
				// stores a new function
				odb.Store(new NeoDatis.Odb.Test.VO.Login.User("uncommited user", "uncommied email"
					, new NeoDatis.Odb.Test.VO.Login.Profile("uncommiedt profile", new NeoDatis.Odb.Test.VO.Login.Function
					("uncommited function"))));
				NeoDatis.Odb.Objects<User> users = odb.GetObjects<User>();
				AssertEquals(5, users.Count);
				NeoDatis.Odb.Objects<Function> functions = odb.GetObjects<Function>();
				AssertEquals(5, functions.Count);
				NeoDatis.Odb.Objects<Profile> profiles = odb.GetObjects<Profile>();
				AssertEquals(5, profiles.Count);
			}
			finally
			{
				if (odb != null)
				{
					odb.Close();
					DeleteBase(BaseName);
				}
			}
		}

		/// <summary>Test select objects that are not yet commited.</summary>
		/// <remarks>
		/// Test select objects that are not yet commited. It also test the meta
		/// model class reference for in transaction class creation
		/// </remarks>
		/// <exception cref="System.Exception">System.Exception</exception>
		[Test]
        public virtual void TestSelectUnCommitedObject3()
		{
			DeleteBase(BaseName);
			// Create instance
			NeoDatis.Odb.Test.VO.Sport.Sport sport = new NeoDatis.Odb.Test.VO.Sport.Sport("volley-ball"
				);
			NeoDatis.Odb.ODB odb = null;
			try
			{
				// Open the database
				odb = Open(BaseName);
				// Store the object
				odb.Store(sport);
			}
			finally
			{
				if (odb != null)
				{
					// Close the database
					odb.Close();
				}
			}
			try
			{
				// Open the database
				odb = Open(BaseName);
				// Let's insert a tennis player
				NeoDatis.Odb.Test.VO.Sport.Player agassi = new NeoDatis.Odb.Test.VO.Sport.Player(
					"Andr√© Agassi", new System.DateTime(), new NeoDatis.Odb.Test.VO.Sport.Sport("Tennis"
					));
				odb.Store(agassi);
				NeoDatis.Odb.Core.Query.IQuery query = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
					(typeof(Player), NeoDatis.Odb.Core.Query.Criteria.Where
					.Equal("favoriteSport.name", "volley-ball"));
				NeoDatis.Odb.Objects<Player> players = odb.GetObjects<Player>(query);
				Println("\nStep 4 : Players of Voller-ball");
				int i = 1;
				// display each object
				while (players.HasNext())
				{
					Println((i++) + "\t: " + players.Next());
				}
			}
			finally
			{
				if (odb != null)
				{
					// Close the database
					odb.Close();
				}
			}
			DeleteBase(BaseName);
		}

		/// <summary>Test select objects that are not yet commited</summary>
		/// <exception cref="System.Exception">System.Exception</exception>
		[Test]
        public virtual void TestSelectUnCommitedObject4()
		{
			DeleteBase(BaseName);
			// Create instance
			NeoDatis.Odb.Test.VO.Sport.Sport sport = new NeoDatis.Odb.Test.VO.Sport.Sport("volley-ball"
				);
			NeoDatis.Odb.ODB odb = null;
			try
			{
				// Open the database
				odb = Open(BaseName);
				// Store the object
				odb.Store(sport);
			}
			finally
			{
				if (odb != null)
				{
					// Close the database
					odb.Close();
				}
			}
			// Create instance
			NeoDatis.Odb.Test.VO.Sport.Sport volleyball = new NeoDatis.Odb.Test.VO.Sport.Sport
				("volley-ball");
			// Create 4 players
			NeoDatis.Odb.Test.VO.Sport.Player player1 = new NeoDatis.Odb.Test.VO.Sport.Player
				("olivier", new System.DateTime(), volleyball);
			NeoDatis.Odb.Test.VO.Sport.Player player2 = new NeoDatis.Odb.Test.VO.Sport.Player
				("pierre", new System.DateTime(), volleyball);
			NeoDatis.Odb.Test.VO.Sport.Player player3 = new NeoDatis.Odb.Test.VO.Sport.Player
				("elohim", new System.DateTime(), volleyball);
			NeoDatis.Odb.Test.VO.Sport.Player player4 = new NeoDatis.Odb.Test.VO.Sport.Player
				("minh", new System.DateTime(), volleyball);
			// Create two teams
			NeoDatis.Odb.Test.VO.Sport.Team team1 = new NeoDatis.Odb.Test.VO.Sport.Team("Paris"
				);
			NeoDatis.Odb.Test.VO.Sport.Team team2 = new NeoDatis.Odb.Test.VO.Sport.Team("Montpellier"
				);
			// Set players for team1
			team1.AddPlayer(player1);
			team1.AddPlayer(player2);
			// Set players for team2
			team2.AddPlayer(player3);
			team2.AddPlayer(player4);
			// Then create a volley ball game for the two teams
			NeoDatis.Odb.Test.VO.Sport.Game game = new NeoDatis.Odb.Test.VO.Sport.Game(new System.DateTime
				(), volleyball, team1, team2);
			odb = null;
			try
			{
				// Open the database
				odb = Open(BaseName);
				// Store the object
				odb.Store(game);
			}
			finally
			{
				if (odb != null)
				{
					// Close the database
					odb.Close();
				}
			}
			try
			{
				// Open the database
				odb = Open(BaseName);
				NeoDatis.Odb.Core.Query.IQuery query = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
					(typeof(NeoDatis.Odb.Test.VO.Sport.Player), NeoDatis.Odb.Core.Query.Criteria.Where
					.Equal("name", "olivier"));
				NeoDatis.Odb.Objects<Player> players = odb.GetObjects<Player>(query);
				Println("\nStep 3 : Players with name olivier");
				int i = 1;
				// display each object
				while (players.HasNext())
				{
					Println((i++) + "\t: " + players.Next());
				}
			}
			finally
			{
				if (odb != null)
				{
					// Close the database
					odb.Close();
				}
			}
			try
			{
				// Open the database
				odb = Open(BaseName);
				// Let's insert a tennis player
				NeoDatis.Odb.Test.VO.Sport.Player agassi = new NeoDatis.Odb.Test.VO.Sport.Player(
					"Andr√© Agassi", new System.DateTime(), new NeoDatis.Odb.Test.VO.Sport.Sport("Tennis"
					));
				NeoDatis.Odb.OID oid = odb.Store(agassi);
				NeoDatis.Odb.Core.Query.IQuery query = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
					(typeof(NeoDatis.Odb.Test.VO.Sport.Player), NeoDatis.Odb.Core.Query.Criteria.Where
					.Equal("favoriteSport.name", "volley-ball"));
				NeoDatis.Odb.Objects<Player> players = odb.GetObjects<Player>(query);
				Println("\nStep 4 : Players of Voller-ball");
				int i = 1;
				// display each object
				while (players.HasNext())
				{
					Println((i++) + "\t: " + players.Next());
				}
			}
			finally
			{
				if (odb != null)
				{
					// Close the database
					odb.Close();
				}
			}
			DeleteBase(BaseName);
		}
	}
}
