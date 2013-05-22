namespace NeoDatis.Odb.Test.Tutorial
{
	public class Team
	{
		private string name;

		private System.Collections.IList players;

		public Team(string name)
		{
			this.name = name;
			players = new System.Collections.ArrayList();
		}

		/// <returns>the name</returns>
		public virtual string GetName()
		{
			return name;
		}

		/// <param name="name">the name to set</param>
		public virtual void SetName(string name)
		{
			this.name = name;
		}

		/// <returns>the players</returns>
		public virtual System.Collections.IList GetPlayers()
		{
			return players;
		}

		/// <param name="players">the players to set</param>
		public virtual void SetPlayers(System.Collections.IList players)
		{
			this.players = players;
		}

		public virtual void AddPlayer(NeoDatis.Odb.Test.Tutorial.Player player)
		{
			players.Add(player);
		}

		public override string ToString()
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			buffer.Append("Team ").Append(name).Append(" ").Append(players);
			return buffer.ToString();
		}
	}
}
