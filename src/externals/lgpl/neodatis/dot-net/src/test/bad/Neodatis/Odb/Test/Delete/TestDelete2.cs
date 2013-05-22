using NeoDatis.Odb.Test.VO.Arraycollectionmap.Catalog;
using NeoDatis.Odb.Test.VO.Login;
namespace NeoDatis.Odb.Test.Delete
{
	public class TestDelete2 : NeoDatis.Odb.Test.ODBTest
	{
		public virtual void TestDeleteListElements()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName);
			NeoDatis.Odb.Test.VO.Login.Profile p = new NeoDatis.Odb.Test.VO.Login.Profile("name"
				);
			p.AddFunction(new NeoDatis.Odb.Test.VO.Login.Function("f1"));
			p.AddFunction(new NeoDatis.Odb.Test.VO.Login.Function("f2"));
			p.AddFunction(new NeoDatis.Odb.Test.VO.Login.Function("3"));
			odb.Store(p);
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Objects objects = odb.GetObjects<Profile>();
			while (objects.HasNext())
			{
				NeoDatis.Odb.Test.VO.Login.Profile profile = objects.Next();
				System.Collections.IList functions = profile.GetFunctions();
				for (int j = 0; j < functions.Count; j++)
				{
					odb.Delete(functions[j]);
				}
				odb.Delete(profile);
			}
			odb.Close();
		}

		public virtual void TestDeleteListElements2()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName);
			Catalog catalog = new Catalog
				("Fnac");
			ProductCategory books = new ProductCategory
				("Books");
			books.GetProducts().Add(new Product
				("Book1", new System.Decimal(10.1)));
			books.GetProducts().Add(new Product
				("Book2", new System.Decimal(10.2)));
			books.GetProducts().Add(new Product
				("Book3", new System.Decimal(10.3)));
			ProductCategory computers = new ProductCategory
				("Computers");
			computers.GetProducts().Add(new Product
				("MacBook", new System.Decimal(1300.1)));
			computers.GetProducts().Add(new Product
				("BookBookPro", new System.Decimal(2000.2)));
			computers.GetProducts().Add(new Product
				("MacMini", new System.Decimal(1000.3)));
			catalog.GetCategories().Add(books);
			catalog.GetCategories().Add(computers);
			odb.Store(catalog);
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Objects objects = odb.GetObjects(typeof(Catalog
				));
			Println(objects.Count + " catalog(s)");
			while (objects.HasNext())
			{
				Catalog c = (Catalog
					)objects.Next();
				System.Collections.IList pCategories = c.GetCategories();
				Println(c.GetCategories().Count + " product categories");
				for (int j = 0; j < pCategories.Count; j++)
				{
					ProductCategory pc = (ProductCategory
						)pCategories[j];
					Println("\tProduct Category : " + pc.GetName() + " : " + pc.GetProducts().Count +
						 " products");
					for (int k = 0; k < pc.GetProducts().Count; k++)
					{
						Product p = pc.GetProducts()[k];
						Println("\t\tProduct " + p.GetName());
						odb.Delete(p);
					}
					odb.Delete(pc);
				}
				odb.Delete(c);
			}
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Objects<Catalog> catalogs = odb.GetObjects<Catalog>();
			NeoDatis.Odb.Objects<ProductCategory> productCategories = odb.GetObjects<ProductCategory>();
			NeoDatis.Odb.Objects<Product> products = odb.GetObjects<Product>();
			AssertTrue(catalogs.Count==0);
			AssertTrue(productCategories.Count==0);
			AssertTrue(products.Count==0);
			DeleteBase(baseName);
		}
	}
}
