using NUnit.Framework;
namespace NeoDatis.Odb.Test.List.Update
{
	[TestFixture]
    public class TestUpdateList : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test1()
		{
			string file = "testeul.neodatis";
			DeleteBase(file);
			NeoDatis.Odb.Test.List.Update.DadosUsuario dadosUsuario = new NeoDatis.Odb.Test.List.Update.DadosUsuario
				();
			dadosUsuario.SetNome("Olivier");
			dadosUsuario.SetLogin("olivier");
			dadosUsuario.SetEmail("olivier@neodatis.org");
			dadosUsuario.SetOid("oid");
			System.Collections.IList l = new System.Collections.ArrayList();
			l.Add(new NeoDatis.Odb.Test.List.Update.Publicacao("p1", "Texto 1"));
			dadosUsuario.SetPublicados(l);
			NeoDatis.Odb.ODB odb = null;
			try
			{
				odb = Open(file);
				odb.Store(dadosUsuario);
			}
			finally
			{
				if (odb != null)
				{
					odb.Close();
				}
			}
			try
			{
				odb = Open(file);
				NeoDatis.Odb.Objects<DadosUsuario> l2 = odb.GetObjects<DadosUsuario>();
				Println(l2);
				NeoDatis.Odb.Test.List.Update.DadosUsuario du = (NeoDatis.Odb.Test.List.Update.DadosUsuario
					)l2.GetFirst();
				du.GetPublicados().Add(new NeoDatis.Odb.Test.List.Update.Publicacao("p2", "Texto2"
					));
				odb.Store(du);
			}
			finally
			{
				if (odb != null)
				{
					odb.Close();
				}
			}
			try
			{
				odb = Open(file);
				NeoDatis.Odb.Objects<DadosUsuario> l2 = odb.GetObjects<DadosUsuario>();
				Println(l2);
				NeoDatis.Odb.Test.List.Update.DadosUsuario du = (NeoDatis.Odb.Test.List.Update.DadosUsuario
					)l2.GetFirst();
				Println(du.GetPublicados());
				AssertEquals(2, du.GetPublicados().Count);
			}
			finally
			{
				if (odb != null)
				{
					odb.Close();
				}
			}
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test2()
		{
			string file = "testeul.neodatis";
			DeleteBase(file);
			NeoDatis.Odb.Test.List.Update.DadosUsuario dadosUsuario = new NeoDatis.Odb.Test.List.Update.DadosUsuario
				();
			dadosUsuario.SetNome("Olivier");
			dadosUsuario.SetLogin("olivier");
			dadosUsuario.SetEmail("olivier@neodatis.org");
			dadosUsuario.SetOid("oid");
			System.Collections.IList l = new System.Collections.ArrayList();
			l.Add(new NeoDatis.Odb.Test.List.Update.Publicacao("p0", "Texto0"));
			dadosUsuario.SetPublicados(l);
			NeoDatis.Odb.ODB odb = null;
			try
			{
				odb = Open(file);
				odb.Store(dadosUsuario);
			}
			finally
			{
				if (odb != null)
				{
					odb.Close();
				}
			}
			int size = 100;
			for (int i = 0; i < size; i++)
			{
				try
				{
					odb = Open(file);
					NeoDatis.Odb.Objects<DadosUsuario> l2 = odb.GetObjects<DadosUsuario>();
					// println(l2);
					NeoDatis.Odb.Test.List.Update.DadosUsuario du = (NeoDatis.Odb.Test.List.Update.DadosUsuario
						)l2.GetFirst();
					du.GetPublicados().Add(new NeoDatis.Odb.Test.List.Update.Publicacao("p" + (i + 1)
						, "Texto" + (i + 1)));
					odb.Store(du);
				}
				finally
				{
					if (odb != null)
					{
						odb.Close();
					}
				}
			}
			try
			{
				odb = Open(file);
				NeoDatis.Odb.Objects<DadosUsuario> l2 = odb.GetObjects<DadosUsuario>();
				Println(l2);
				NeoDatis.Odb.Test.List.Update.DadosUsuario du = (NeoDatis.Odb.Test.List.Update.DadosUsuario
					)l2.GetFirst();
				Println(du.GetPublicados());
				AssertEquals(size + 1, du.GetPublicados().Count);
				for (int i = 0; i < size + 1; i++)
				{
					NeoDatis.Odb.Test.List.Update.Publicacao p = (NeoDatis.Odb.Test.List.Update.Publicacao
						)du.GetPublicados()[i];
					AssertEquals("Texto" + (i), p.GetTexto());
					AssertEquals("p" + (i), p.GetName());
				}
			}
			finally
			{
				if (odb != null)
				{
					odb.Close();
				}
			}
		}
	}
}
