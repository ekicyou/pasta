using NUnit.Framework;
namespace NeoDatis.Odb.Test.VO.Sport
{
	public class Player
	{
		private string name;

		private System.DateTime birthDate;

		private NeoDatis.Odb.Test.VO.Sport.Sport favoriteSport;

		public Player(string name, System.DateTime birthDate, NeoDatis.Odb.Test.VO.Sport.Sport
			 favoriteSport)
		{
			this.name = name;
			this.birthDate = birthDate;
			this.favoriteSport = favoriteSport;
		}

		public virtual System.DateTime GetBirthDate()
		{
			return birthDate;
		}

		public virtual void SetBirthDate(System.DateTime birthDate)
		{
			this.birthDate = birthDate;
		}

		public virtual NeoDatis.Odb.Test.VO.Sport.Sport GetFavoriteSport()
		{
			return favoriteSport;
		}

		public virtual void SetFavoriteSport(NeoDatis.Odb.Test.VO.Sport.Sport favoriteSport
			)
		{
			this.favoriteSport = favoriteSport;
		}

		public virtual string GetName()
		{
			return name;
		}

		public virtual void SetName(string name)
		{
			this.name = name;
		}

		public override string ToString()
		{
			return name;
		}
	}
}
