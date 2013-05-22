namespace NeoDatis.Odb.Test.VO.Arraycollectionmap.Catalog
{
	/// <author>olivier</author>
	public class Product
	{
		private string name;

		private System.Decimal price;

		public Product(string name, System.Decimal price) : base()
		{
			this.name = name;
			this.price = price;
		}

		public virtual string GetName()
		{
			return name;
		}

		public virtual void SetName(string name)
		{
			this.name = name;
		}

		public virtual System.Decimal GetPrice()
		{
			return price;
		}

		public virtual void SetPrice(System.Decimal price)
		{
			this.price = price;
		}
	}
}
