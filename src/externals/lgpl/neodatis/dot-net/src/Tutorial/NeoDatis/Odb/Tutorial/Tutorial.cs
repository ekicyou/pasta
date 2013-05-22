using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using NeoDatis.Odb;
using NeoDatis.Odb.Core.Query;
using NeoDatis.Odb.Impl.Core.Query.Criteria;
using NeoDatis.Odb.Core.Query.Criteria;
using System.IO;

namespace Neodatis.Odb.Tutorial
{
    class TutorialOdb
    {
        public const string ODB_NAME = "tutorial1.neodatis";
        public const string ODB_NAME_2 = "tutorial1bis.neodatis";


        /// <summary>
        ///  How to insert an object in NeoDatis database
        /// </summary>
        public void Step1()
        {
            // Create instance
            Sport sport = new Sport("volley-ball");

            ODB odb = null;

            try
            {
                // Open the database
                odb = ODBFactory.Open(ODB_NAME);

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
        }

        /// <summary>
        /// How to insert a complex object in NeoDatis Database
        /// </summary>
        public void Step2()
        {

            // Create instance
            Sport volleyball = new Sport("volley-ball");

            // Create 4 players
            Player player1 = new Player("olivier", new DateTime(), volleyball);
            Player player2 = new Player("pierre", new DateTime(), volleyball);
            Player player3 = new Player("elohim", new DateTime(), volleyball);
            Player player4 = new Player("minh", new DateTime(), volleyball);

            // Create two teams
            Team team1 = new Team("Paris");
            Team team2 = new Team("Montpellier");

            // Set players for team1
            team1.AddPlayer(player1);
            team1.AddPlayer(player2);

            // Set players for team2
            team2.AddPlayer(player3);
            team2.AddPlayer(player4);

            // Then create a volley ball game for the two teams
            Game game = new Game(new DateTime(), volleyball, team1, team2);

            ODB odb = null;

            try
            {
                // Open the database
                odb = ODBFactory.Open(ODB_NAME);

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
        }

	/// <summary>
	/// How to retrieve objects using Criteria queries
	/// </summary>
	public void Step3()  {
		ODB odb = null;

		try {
			// Open the database
			odb = ODBFactory.Open(ODB_NAME);
			IQuery query = new CriteriaQuery(Where.Equal("name", "olivier"));
			Objects<Player> players = odb.GetObjects<Player>(query);

			Console.WriteLine("\nStep 3 : Players with name olivier");

			int i = 1;
			// display each object
			while (players.HasNext()) {
				Console.WriteLine((i++) + "\t: " + players.Next());
			}
		} finally {
			if (odb != null) {
				// Close the database
				odb.Close();
			}
		}
	}

    /// <summary>
    /// How to retrieve objects using Criteria queries traversing relations
    /// </summary>
    public void Step4()  {

		ODB odb = null;

		try {
			// Open the database
			odb = ODBFactory.Open(ODB_NAME);
			// Let's insert a tennis player
			Player agassi = new Player("Andr\u00E9 Agassi", new DateTime(), new Sport("Tennis"));
			odb.Store(agassi);

			IQuery query = new CriteriaQuery(typeof(Player), Where.Equal("favoriteSport.name", "volley-ball"));

			Objects<Player> players = odb.GetObjects<Player>(query);

			Console.WriteLine("\nStep 4 : Players of Voller-ball");

			int i = 1;
			// display each object
			while (players.HasNext()) {
				Console.WriteLine((i++) + "\t: " + players.Next());
			}
		} finally {
			if (odb != null) {
				// Close the database
				odb.Close();
			}
		}
	}

    /// <summary>
    /// How to retrieve objects using Criteria queries using object
    /// </summary>
    public void Step5()  {

		ODB odb = null;

		try {
			// Open the database
			odb = ODBFactory.Open(ODB_NAME);
			// retrieve the volley ball sport object
			IQuery query = new CriteriaQuery(typeof(Sport), Where.Equal("name", "volley-ball"));
			Sport volleyBall = odb.GetObjects<Sport>(query).GetFirst();

			// Now build a query to get all players that play volley ball, using
			// the volley ball object
			query = new CriteriaQuery( Where.Equal("favoriteSport", volleyBall));

			Objects<Player> players = odb.GetObjects<Player>(query);

			Console.WriteLine("\nStep 5: Players of Voller-ball");

			int i = 1;
			// display each object
			while (players.HasNext()) {
				Console.WriteLine((i++) + "\t: " + players.Next());
			}

		} finally {
			if (odb != null) {
				// Close the database
				odb.Close();
			}
		}
	}

    /**
     * How to retrieve objects using Criteria queries with Or
     * 
     */
    /// <summary>
    /// How to retrieve objects using Criteria queries using object
    /// </summary>
    public void Step6()
    {
		ODB odb = null;

		try {
			// Open the database
			odb = ODBFactory.Open(ODB_NAME);
			IQuery query = new CriteriaQuery(typeof(Player), Where.Or().Add(Where.Equal("favoriteSport.name", "volley-ball")).Add(
					Where.Like("favoriteSport.name", "%nnis")));

			Objects<Player> players = odb.GetObjects<Player>(query);

			Console.WriteLine("\nStep 6 : Volley-ball and Tennis Players");

			int i = 1;
			// display each object
			while (players.HasNext()) {
				Console.WriteLine((i++) + "\t: " + players.Next());
			}
		} finally {
			if (odb != null) {
				// Close the database
				odb.Close();
			}
		}
	}

    /**
     * How to retrieve objects using Criteria queries with Not
     * 
     */
    /// <summary>
    /// How to retrieve objects using Criteria queries using object
    /// </summary>
    public void Step7()
    {
		ODB odb = null;

		try {
			// Open the database
			odb = ODBFactory.Open(ODB_NAME);
			IQuery query = new CriteriaQuery(typeof(Player), Where.Not(Where.Equal("favoriteSport.name", "volley-ball")));

			Objects<Player> players = odb.GetObjects<Player>(query);

			Console.WriteLine("\nStep 7 : Players that don't play Volley-ball");

			int i = 1;
			// display each object
			while (players.HasNext()) {
				Console.WriteLine((i++) + "\t: " + players.Next());
			}

		} finally {
			if (odb != null) {
				// Close the database
				odb.Close();
			}
		}
	}

   
    /// <summary>
    /// How to retrieve objects using Native queries 
    /// </summary>
    /// 
    /*
    public void step8()  {
        ODB odb = null;

        try {
            // Open the database
            odb = ODBFactory.Open(ODB_NAME);
            IQuery query = new SimpleNativeQuery() {  public boolean match(Player player) {
                    return player.getFavoriteSport().getName().toLowerCase().startsWith("volley");
                }
            };

            Objects<Player> players = odb.GetObjects<Player>(query);

            Console.WriteLine("\nStep 8 bis: Players that play Volley-ball");

            int i = 1;
            // display each object
            while (players.HasNext()) {
                Console.WriteLine((i++) + "\t: " + players.Next());
            }

        } finally {
            if (odb != null) {
                // Close the database
                odb.Close();
            }
        }
    }
        */
    /**
     * Native query with Objects,
     * 
     *
    /// <summary>
    /// How to retrieve objects using Criteria queries using object
    /// </summary>
    public void step9()  {
        ODB odb = null;

        try {
            // Open the database
            odb = ODBFactory.Open(ODB_NAME);

            // first retrieve the player Minh
            IQuery query = new CriteriaQuery(typeof(Player), Where.Equal("name", "minh"));
            Player minh = (Player) odb.GetObjects(query).getFirst();

            // builds a query to get all teams where mihn plays
            query = new CriteriaQuery(typeof(Team), Where.contain("players", minh));
            Objects teams = odb.GetObjects(query);

            Console.WriteLine("\nStep 9: Team where minh plays");

            int i = 1;
            // display each object
            while (teams.HasNext()) {
                Console.WriteLine((i++) + "\t: " + teams.Next());
            }

        } finally {
            if (odb != null) {
                // Close the database
                odb.Close();
            }
        }
    }*/


    /// <summary>
    /// How to retrieve objects using order by
    /// </summary>
    public void Step10()  {
		ODB odb = null;

		try {
			// Open the database
			odb = ODBFactory.Open(ODB_NAME);
			IQuery query = new CriteriaQuery(typeof(Player));
			query.OrderByAsc("name");

			Objects<Player> players = odb.GetObjects<Player>(query);

			Console.WriteLine("\nStep 10: Players ordered by name asc");

			int i = 1;
			// display each object
			while (players.HasNext()) {
				Console.WriteLine((i++) + "\t: " + players.Next());
			}

			query.OrderByDesc("name");

			players = odb.GetObjects<Player>(query);

			Console.WriteLine("\nStep 10: Players ordered by name desc");

			i = 1;
			// display each object
			while (players.HasNext()) {
				Console.WriteLine((i++) + "\t: " + players.Next());
			}

		} finally {
			if (odb != null) {
				odb.Close();
			}
		}
	}

    /// <summary>
    /// Using Indexes
    /// </summary>
    public void Step11()  {
        // Open the database
        ODB odb = null;

        try {
            odb = ODBFactory.Open(ODB_NAME);

            String[] fieldNames = { "name" };
            odb.GetClassRepresentation(typeof(Sport)).AddUniqueIndexOn("sport-index", fieldNames, true);
            odb.Close();

            odb = ODBFactory.Open(ODB_NAME);
            IQuery query = new CriteriaQuery(typeof(Sport), Where.Equal("name", "volley-ball"));

            Objects<Sport> sports = odb.GetObjects<Sport>(query);

            Console.WriteLine("\nStep 11 : Using index");

            int i = 1;
            // display each object
            while (sports.HasNext()) {
                Console.WriteLine((i++) + "\t: " + sports.Next());
            }

        } finally {
            if (odb != null) {
                // Close the database
                odb.Close();
            }
        }
    }
    /// <summary>
    /// How to update objects
    /// </summary>
    public void Step12()  {
		ODB odb = null;

		try {
			// Open the database
			odb = ODBFactory.Open(ODB_NAME);
			IQuery query = new CriteriaQuery(typeof(Sport), Where.Equal("name", "volley-ball"));

			Objects<Sport> sports = odb.GetObjects<Sport>(query);

			// Gets the first sport (there is only one!)
			Sport volley = sports.GetFirst();

			// Changes the name
			volley.SetName("Beach-Volley");

			// Actually updates the object
			odb.Store(volley);

			// Commits the changes
			odb.Close();

			odb = ODBFactory.Open(ODB_NAME);
			// Now query the database to check the change
			sports = odb.GetObjects<Sport>();

			Console.WriteLine("\nStep 12 : Updating sport");

			int i = 1;
			// display each object
			while (sports.HasNext()) {
				Console.WriteLine((i++) + "\t: " + sports.Next());
			}

		} finally {
			if (odb != null) {
				// Close the database
				odb.Close();
			}
		}
    }


    /// <summary>
    /// How to delete Objects
    /// </summary>
    public void Step13()  {

		ODB odb = null;

		try {
			// Open the database
			odb = ODBFactory.Open(ODB_NAME);
            IQuery queryAll = new CriteriaQuery(typeof(Player));
			IQuery query = new CriteriaQuery(typeof(Player), Where.Like("name", "%Agassi"));

            Objects<Player> players = odb.GetObjects<Player>(queryAll);
			players = odb.GetObjects<Player>(query);

			// Gets the first player (there is only one!)
			Player agassi = players.GetFirst();

			odb.Delete(agassi);

			odb.Close();

			odb = ODBFactory.Open(ODB_NAME);
			// Now query the databas eto check the change
			players = odb.GetObjects<Player>();

			Console.WriteLine("\nStep 13 : Deleting Agassi");

			int i = 1;
			// display each object
			while (players.HasNext()) {
				Console.WriteLine((i++) + "\t: " + players.Next());
			}

		} finally {
			if (odb != null) {
				// Close the database
				odb.Close();
			}
		}
	}

    /// <summary>
    /// How to delete an object using OID
    /// </summary>
	public void Step14()  {
		ODB odb = null;

		try {
			// Open the database
			odb = ODBFactory.Open(ODB_NAME);

			Sport tennis = odb.GetObjects<Sport>(new CriteriaQuery(typeof(Sport), Where.Equal("name", "Tennis"))).GetFirst();
			// Firts re-create Agassi player - it has been deleted in step 13
			Player agassi = new Player("Andr\u00E9 Agassi", new DateTime(), tennis);
			odb.Store(agassi);
			odb.Commit();

			IQuery query = new CriteriaQuery(typeof(Player), Where.Like("name", "%Agassi"));

			Objects<Player> players = odb.GetObjects<Player>(query);

			// Gets the first player (there is only one!)
			agassi = players.GetFirst();
			OID agassiId = odb.GetObjectId(agassi);

			odb.DeleteObjectWithId(agassiId);

			odb.Close();

			odb = ODBFactory.Open(ODB_NAME);
			// Now query the databas eto check the change
			players = odb.GetObjects<Player>();

			Console.WriteLine("\nStep 14 : Deleting players");

			int i = 1;
			// display each object
			while (players.HasNext()) {
				Console.WriteLine((i++) + "\t: " + players.Next());
			}

		} finally {
			if (odb != null) {
				// Close the database
				odb.Close();
			}
		}
	}


    /**
     * exporting to XML
     * 
     *
    /// <summary>
    /// How to export the database
    /// </summary>

    public void Step15()  {
        ODB odb = null;

        try {
            // Open the database
            odb = ODBFactory.Open(ODB_NAME);
            // Creates the exporter
            XMLExporter exporter = new XMLExporter(odb);

            // Actually export to current directory into the sports.xml file
            exporter.export(".", "sports.xml");
        } finally {
            if (odb != null) {
                // Close the database
                odb.Close();
            }
        }
        Console.WriteLine("\nStep 15 : exporting database to sports.xml");
    }
    */
    /*
    /// <summary>
    /// How to import a database
    /// </summary>
    public void Step16()  {
        ODB odb = null;

        try {
            // Delete database first
            File.Delete("imported-" + ODB_NAME);

            // Open a database to receive imported data
            odb = ODBFactory.Open("imported-" + ODB_NAME);
            // Creates the exporter
            XMLImporter importer = new XMLImporter(odb);

            // Actually import data from sports.xml file
            importer.importFile(".", "sports.xml");

            // Closes the database
            odb.Close();

            // Re Open the database
            odb = ODBFactory.Open("imported-" + ODB_NAME);
            // Now query the databas eto check the change
            Objects players = odb.GetObjects(typeof(Player));

            Console.WriteLine("\nStep 16 : getting players of imported database");

            int i = 1;
            // display each object
            while (players.HasNext()) {
                Console.WriteLine((i++) + "\t: " + players.Next());
            }

        } catch (Exception e) {
            Console.WriteLine(e);
        } finally {
            if (odb != null) {
                // Close the database
                odb.Close();
            }
        }
    }
     * */
    /// <summary>
    /// Database protected by user and password
    /// </summary>
	public void Step17()  {
		ODB odb = null;

		try {
			// Open the database
			odb = ODBFactory.Open(ODB_NAME_2, "user", "password");
			odb.Store(new Sport("Tennis"));
			// Commits the changes
			odb.Close();

			try {
				// try to Open the database without user/password
				odb = ODBFactory.Open(ODB_NAME_2);
			} catch (ODBAuthenticationRuntimeException e) {
				Console.WriteLine("\nStep 17 : invalid user/password : database could not be Opened");
			}
			// then Open the database with correct user/password
			odb = ODBFactory.Open(ODB_NAME_2, "user", "password");
			Console.WriteLine("\nStep 17 : user/password : database Opened");
		} finally {
			if (odb != null) {
				// Close the database
				odb.Close();
			}
		}
	}

	public void DisplaySports(String label1)  {
		// Open the database
		ODB odb = null;

		try {
			odb = ODBFactory.Open(ODB_NAME);
			// Get all object of type clazz
			Objects<Sport> objects = odb.GetObjects<Sport>();

			Console.WriteLine("\nSports : " + label1);

			int i = 1;
			// display each object
			while (objects.HasNext()) {
				Console.WriteLine((i++) + "\t: " + objects.Next());
			}

		} finally {
			if (odb != null) {
				// Close the database
				odb.Close();
			}
		}
	}

    public void DisplayTeams(String label1)
    {
        // Open the database
        ODB odb = null;

        try
        {
            odb = ODBFactory.Open(ODB_NAME);
            // Get all object of type clazz
            Objects<Team> objects = odb.GetObjects<Team>();

            Console.WriteLine("\nTeams : " + label1 );

            int i = 1;
            // display each object
            while (objects.HasNext())
            {
                Console.WriteLine((i++) + "\t: " + objects.Next());
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
    }

    public void DisplayPlayers(String label1)
    {
        // Open the database
        ODB odb = null;

        try
        {
            odb = ODBFactory.Open(ODB_NAME);
            // Get all object of type clazz
            Objects<Player> objects = odb.GetObjects<Player>();

            Console.WriteLine("\n Players : " + label1);

            int i = 1;
            // display each object
            while (objects.HasNext())
            {
                Console.WriteLine((i++) + "\t: " + objects.Next());
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
    }

    public void DisplayGames(String label1)
    {
        // Open the database
        ODB odb = null;

        try
        {
            odb = ODBFactory.Open(ODB_NAME);
            // Get all object of type clazz
            Objects<Game> objects = odb.GetObjects<Game>();

            Console.WriteLine("\n Games : " + label1);

            int i = 1;
            // display each object
            while (objects.HasNext())
            {
                Console.WriteLine((i++) + "\t: " + objects.Next());
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
    }



    /// <summary>
    /// Main
    /// </summary>

        static void Main(string[] args)
        {
            File.Delete(ODB_NAME);
		    TutorialOdb tutorial1 = new TutorialOdb();

		    tutorial1.Step1();
		    tutorial1.DisplaySports("Step 1");

            File.Delete(ODB_NAME);

		    tutorial1.Step2();
		    tutorial1.DisplayGames("Step 2");
		    tutorial1.DisplayTeams("Step 2");
		    tutorial1.DisplayPlayers("Step 2");
		    tutorial1.DisplaySports("Step 2");

		    tutorial1.Step3();

		    tutorial1.Step4();
		    tutorial1.Step5();
		    tutorial1.Step6();
		    tutorial1.Step7();
		    //tutorial1.Step8();
		    //tutorial1.step9();
		    tutorial1.Step10();
		    //tutorial1.step11();
		    tutorial1.Step12();
		    tutorial1.Step13();
    		
		    //tutorial1.step14();
		    //tutorial1.step15();
		    //tutorial1.step16();
		    tutorial1.Step17();

            Console.ReadLine();
        }
    }
}
