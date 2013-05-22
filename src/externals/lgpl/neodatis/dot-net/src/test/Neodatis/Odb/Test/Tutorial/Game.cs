using NUnit.Framework;
namespace NeoDatis.Odb.Test.Tutorial
{
	public class Game
	{
		private System.DateTime when;

		private NeoDatis.Odb.Test.Tutorial.Sport sport;

		private NeoDatis.Odb.Test.Tutorial.Team team1;

		private NeoDatis.Odb.Test.Tutorial.Team team2;

		private string result;

		public Game(System.DateTime when, NeoDatis.Odb.Test.Tutorial.Sport sport, NeoDatis.Odb.Test.Tutorial.Team
			 team1, NeoDatis.Odb.Test.Tutorial.Team team2)
		{
			this.when = when;
			this.sport = sport;
			this.team1 = team1;
			this.team2 = team2;
		}

		public virtual string GetResult()
		{
			return result;
		}

		public virtual void SetResult(string result)
		{
			this.result = result;
		}

		public virtual NeoDatis.Odb.Test.Tutorial.Sport GetSport()
		{
			return sport;
		}

		public virtual void SetSport(NeoDatis.Odb.Test.Tutorial.Sport sport)
		{
			this.sport = sport;
		}

		public virtual NeoDatis.Odb.Test.Tutorial.Team GetTeam1()
		{
			return team1;
		}

		public virtual void SetTeam1(NeoDatis.Odb.Test.Tutorial.Team team1)
		{
			this.team1 = team1;
		}

		public virtual NeoDatis.Odb.Test.Tutorial.Team GetTeam2()
		{
			return team2;
		}

		public virtual void SetTeam2(NeoDatis.Odb.Test.Tutorial.Team team2)
		{
			this.team2 = team2;
		}

		public override string ToString()
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			buffer.Append(when).Append(" : Game of ").Append(sport).Append(" between ").Append
				(team1.GetName()).Append(" and ").Append(team2.GetName());
			return buffer.ToString();
		}
	}
}
