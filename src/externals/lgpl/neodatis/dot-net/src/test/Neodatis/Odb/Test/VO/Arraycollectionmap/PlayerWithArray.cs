using NUnit.Framework;
namespace NeoDatis.Odb.Test.VO.Arraycollectionmap
{
	public class PlayerWithArray
	{
		private string name;

		private string[] games;

		private int numberOfGames;

		public PlayerWithArray()
		{
		}

		public PlayerWithArray(string name)
		{
			this.name = name;
			this.games = new string[50];
			this.numberOfGames = 0;
		}

		public virtual void AddGame(string gameName)
		{
			games[numberOfGames] = gameName;
			numberOfGames++;
		}

		public virtual string GetGame(int index)
		{
			return games[index];
		}

		public override string ToString()
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			buffer.Append("Name=").Append(name).Append("[");
			for (int i = 0; i < numberOfGames; i++)
			{
				buffer.Append(games[i]).Append(" ");
			}
			buffer.Append("]");
			return buffer.ToString();
		}
	}
}
