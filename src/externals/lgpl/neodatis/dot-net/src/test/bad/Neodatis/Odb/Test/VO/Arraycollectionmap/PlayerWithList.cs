namespace NeoDatis.Odb.Test.VO.Arraycollectionmap
{
	public class PlayerWithList
	{
		private string name;

		private string desc;

		private System.Collections.IList games;

		private int numberOfGames;

		public PlayerWithList()
		{
		}

		public PlayerWithList(string name)
		{
			this.name = name;
			this.games = new System.Collections.ArrayList();
			this.numberOfGames = 0;
		}

		public virtual void AddGame(string gameName)
		{
			games.Add(gameName);
			numberOfGames++;
		}

		public virtual string GetGame(int index)
		{
			return (string)games[index];
		}

		public virtual void SetGames(System.Collections.IList games)
		{
			this.games = games;
		}

		public override string ToString()
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			buffer.Append("Name=").Append(name).Append("[");
			for (int i = 0; i < numberOfGames; i++)
			{
				buffer.Append(GetGame(i)).Append(" ");
			}
			buffer.Append("]");
			return buffer.ToString();
		}
	}
}
