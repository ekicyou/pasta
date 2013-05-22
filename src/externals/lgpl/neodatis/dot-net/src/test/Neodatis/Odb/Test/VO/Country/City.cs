using NUnit.Framework;
namespace NeoDatis.Odb.Test.VO.Country
{
	public class City
	{
		private string name;

		private NeoDatis.Odb.Test.VO.Country.Country country;

		public City()
		{
		}

		public City(string name)
		{
			this.name = name;
		}

		public virtual NeoDatis.Odb.Test.VO.Country.Country GetCountry()
		{
			return country;
		}

		public virtual void SetCountry(NeoDatis.Odb.Test.VO.Country.Country country)
		{
			this.country = country;
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
			return name + " - Country = " + (country != null ? country.GetName() : "null");
		}
	}
}
