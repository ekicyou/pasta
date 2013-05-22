namespace NeoDatis.Odb.Test.VO.Country
{
	public class Country
	{
		private string name;

		private int population;

		private System.Collections.IList cities;

		public Country()
		{
			cities = new System.Collections.ArrayList();
		}

		public Country(string name) : this()
		{
			this.name = name;
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

		public virtual void AddCity(NeoDatis.Odb.Test.VO.Country.City city)
		{
			cities.Add(city);
		}

		public virtual System.Collections.IList GetCities()
		{
			return cities;
		}

		public virtual int GetPopulation()
		{
			return population;
		}

		public virtual void SetPopulation(int population)
		{
			this.population = population;
		}
	}
}
