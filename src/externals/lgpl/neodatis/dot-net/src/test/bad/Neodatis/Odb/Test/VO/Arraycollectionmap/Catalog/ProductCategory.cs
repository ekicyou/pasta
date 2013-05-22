namespace NeoDatis.Odb.Test.VO.Arraycollectionmap.Catalog
{
	/// <author>olivier</author>
	public class ProductCategory
	{
		private string name;

		private System.Collections.Generic.IList<NeoDatis.Odb.Test.VO.Arraycollectionmap.Catalog.Product
			> products;

		public ProductCategory(string name) : base()
		{
			this.name = name;
			products = new System.Collections.Generic.List<NeoDatis.Odb.Test.VO.Arraycollectionmap.Catalog.Product
				>();
		}

		public virtual string GetName()
		{
			return name;
		}

		public virtual void SetName(string name)
		{
			this.name = name;
		}

		public virtual System.Collections.Generic.IList<NeoDatis.Odb.Test.VO.Arraycollectionmap.Catalog.Product
			> GetProducts()
		{
			return products;
		}

		public virtual void SetProducts(System.Collections.Generic.IList<NeoDatis.Odb.Test.VO.Arraycollectionmap.Catalog.Product
			> products)
		{
			this.products = products;
		}
	}
}
