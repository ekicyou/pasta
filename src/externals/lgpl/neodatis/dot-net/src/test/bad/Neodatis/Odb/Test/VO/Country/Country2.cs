namespace NeoDatis.Odb.Test.VO.Country
{
	public class Country2 : NeoDatis.Odb.Test.VO.Country.Country
	{
		private NeoDatis.Odb.Test.VO.Country.City capital;

		public Country2() : base()
		{
		}

		public Country2(string name) : base(name)
		{
		}

		public virtual NeoDatis.Odb.Test.VO.Country.City GetCapital()
		{
			return capital;
		}

		public virtual void SetCapital(NeoDatis.Odb.Test.VO.Country.City capital)
		{
			this.capital = capital;
			AddCity(capital);
		}

		public override string ToString()
		{
			return base.ToString() + " - Capital = " + (capital != null ? capital.GetName() : 
				"null");
		}
	}
}
