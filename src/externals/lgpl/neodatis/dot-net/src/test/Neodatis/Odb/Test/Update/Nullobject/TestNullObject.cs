using NeoDatis.Odb.Impl.Core.Query.Criteria;
using NeoDatis.Odb.Core.Query.Criteria;
using NUnit.Framework;
namespace NeoDatis.Odb.Test.Update.Nullobject
{
	[TestFixture]
    public class TestNullObject : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test1()
		{
			DeleteBase("sict");
			NeoDatis.Odb.ODB odb = Open("sict");
			NeoDatis.Odb.Test.Update.Nullobject.User user = Popula(odb);
			AT at = CreateAT(user);
			odb.Store(at);
			odb.Store(CreateSensor(user, at, 1));
			odb.Store(CreateSensor(user, at, 2));
			odb.Store(CreateSensor(user, at, 3));
			odb.Store(CreateSensor(user, at, 4));
			odb.Close();
			odb = Open("sict");
			NeoDatis.Odb.Objects<AT> ats = odb.GetObjects<AT>();
			int nbAts = ats.Count;
			at = (AT)ats.GetFirst();
			AT newAT = null;
			SensorAT newSensor = null;
			NeoDatis.Odb.Core.Query.IQuery query = new CriteriaQuery(Where.Equal("at.name", at.GetName()));
			query.OrderByAsc("lane");
			NeoDatis.Odb.Objects<SensorAT> sensors = odb.GetObjects<SensorAT>(query);
			Println("Duplicando AT " + at.GetName());
			for (int i = 0; i < 10; i++)
			{
				newAT = DuplicateAT(at, nbAts + i + 1);
				odb.Store(newAT);
				sensors.Reset();
				while (sensors.HasNext())
				{
					newSensor = DuplicateSensor((SensorAT)sensors
						.Next(), newAT);
					odb.Store(newSensor);
				}
			}
			// println("AT " + newAT.getName()+" created");
			odb.Close();
		}

		public static AT CreateAT(NeoDatis.Odb.Test.Update.Nullobject.User
			 user)
		{
			NeoDatis.Odb.Test.Update.Nullobject.Constructor constructor = new NeoDatis.Odb.Test.Update.Nullobject.Constructor
				();
			constructor.SetCreationDate(new System.DateTime());
			constructor.SetName("neodatis");
			constructor.SetDescription("Neodatis");
			AT newAt = new AT
				();
			newAt.SetName("AT1");
			newAt.SetConstructor(constructor);
			newAt.SetCreationDate(new System.DateTime());
			newAt.SetDeleted(false);
			newAt.SetIpAddress("1.1.1.1");
			newAt.SetPhysicalAddress("A01");
			newAt.SetPort(4000);
			newAt.SetStatus(true);
			newAt.SetType("Type1");
			newAt.SetUpdateDate(new System.DateTime());
			newAt.SetUser(user);
			return newAt;
		}

		public static SensorAT CreateSensor(NeoDatis.Odb.Test.Update.Nullobject.User
			 user, AT at, int index)
		{
			SensorAT newSensorAT = new SensorAT
				();
			newSensorAT.SetName(at.GetName() + "-" + index);
			newSensorAT.SetCreationDate(new System.DateTime());
			newSensorAT.SetDeleted(false);
			newSensorAT.SetKm(System.Convert.ToSingle(105.7));
			newSensorAT.SetLane(index);
			newSensorAT.SetState(1);
			newSensorAT.SetStatus(true);
			newSensorAT.SetUpdateDate(new System.DateTime());
			newSensorAT.SetUser(user);
			newSensorAT.SetWay(1);
			newSensorAT.SetAt(at);
			return newSensorAT;
		}

		public static AT DuplicateAT(AT
			 at, int index)
		{
			AT newAt = new AT
				();
			newAt.SetName(at.GetName() + "-" + index);
			newAt.SetConstructor(at.GetConstructor());
			newAt.SetCreationDate(new System.DateTime());
			newAt.SetDeleted(false);
			newAt.SetIpAddress(at.GetIpAddress());
			newAt.SetPhysicalAddress(at.GetPhysicalAddress());
			newAt.SetPort(at.GetPort());
			newAt.SetStatus(true);
			newAt.SetType(at.GetType());
			newAt.SetUpdateDate(new System.DateTime());
			newAt.SetUser(at.GetUser());
			return newAt;
		}

		public static SensorAT DuplicateSensor(SensorAT
			 sensorAT, AT at)
		{
			SensorAT newSensorAT = new SensorAT
				();
			newSensorAT.SetName(at.GetName() + "-" + sensorAT.GetName());
			newSensorAT.SetCreationDate(new System.DateTime());
			newSensorAT.SetDeleted(false);
			newSensorAT.SetKm(sensorAT.GetKm());
			newSensorAT.SetLane(sensorAT.GetLane());
			newSensorAT.SetState(sensorAT.GetState());
			newSensorAT.SetStatus(true);
			newSensorAT.SetUpdateDate(new System.DateTime());
			newSensorAT.SetUser(sensorAT.GetUser());
			newSensorAT.SetWay(sensorAT.GetWay());
			newSensorAT.SetAt(at);
			return newSensorAT;
		}

		/// <exception cref="System.Exception"></exception>
		public static NeoDatis.Odb.Test.Update.Nullobject.User Popula(NeoDatis.Odb.ODB odb
			)
		{
			// cria perfil
			NeoDatis.Odb.Test.Update.Nullobject.Profile profileAdmin = new NeoDatis.Odb.Test.Update.Nullobject.Profile
				("administrador");
			odb.Store(profileAdmin);
			NeoDatis.Odb.Test.Update.Nullobject.Profile profileOper = new NeoDatis.Odb.Test.Update.Nullobject.Profile
				("operador");
			odb.Store(profileOper);
			// cria funcao
			CreateFunctionProfile(odb, profileAdmin, profileOper);
			// cria usuario
			NeoDatis.Odb.Test.Update.Nullobject.User user = new NeoDatis.Odb.Test.Update.Nullobject.User
				();
			user.SetCreationDate(new System.DateTime());
			user.SetDeleted(false);
			user.SetLastLogin(new System.DateTime());
			user.SetLogin("admin");
			user.SetName("Administrador");
			user.SetPassword("trocar");
			user.SetProfileId(profileAdmin);
			user.SetRejectedLogin(0);
			user.SetUpdateDate(new System.DateTime());
			user.SetStatus(true);
			user.SetSessionKey("123456");
			odb.Store(user);
			return user;
		}

		/// <exception cref="System.Exception"></exception>
		public static void CreateFunctionProfile(NeoDatis.Odb.ODB odb, NeoDatis.Odb.Test.Update.Nullobject.Profile
			 admin, NeoDatis.Odb.Test.Update.Nullobject.Profile oper)
		{
			NeoDatis.Odb.Test.Update.Nullobject.Functions function = new NeoDatis.Odb.Test.Update.Nullobject.Functions
				();
			function.SetDescription("Inclus√£o de usu√£rio");
			function.SetName("incluiUsuario");
			function.SetNameUrl("usuario.do/criar");
			function.AddProfile(admin);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Edi√£√£o de Usu√£rio");
			function.SetName("editaUsuario");
			function.SetNameUrl("usuario.do/edit");
			function.AddProfile(admin);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Exclus√£o de Usu√£rio");
			function.SetName("excluiUsuario");
			function.SetNameUrl("usuario.do/excluir");
			function.AddProfile(admin);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Consulta de Usu√£rios");
			function.SetName("listaUsuario");
			function.SetNameUrl("usuario.do/visualizar");
			function.AddProfile(admin);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Controller do usu√£rio");
			function.SetName("usuario");
			function.SetNameUrl("consultaUsuario.do/editar");
			function.AddProfile(admin);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Controller do usu√£rio");
			function.SetName("usuario");
			function.SetNameUrl("consultaUsuario.do/excluir");
			function.AddProfile(admin);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Controller do usu√£rio");
			function.SetName("usuario");
			function.SetNameUrl("consultaUsuario.do/visualizar");
			function.AddProfile(admin);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Controller da senha");
			function.SetName("alteraSenha");
			function.SetNameUrl("alteraSenha.do/editar");
			function.AddProfile(admin);
			function.AddProfile(oper);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Controller da senha");
			function.SetName("alteraSenha");
			function.SetNameUrl("alteraSenha.do");
			function.AddProfile(admin);
			function.AddProfile(oper);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Altera√£√£o de Senha de outros");
			function.SetName("alteraSenhaOutros");
			function.SetNameUrl("alteraSenhaOutros.do/editar");
			function.AddProfile(admin);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Altera√£√£o de Senha de outros");
			function.SetName("alteraSenhaOutros");
			function.SetNameUrl("alteraSenhaOutros.do");
			function.AddProfile(admin);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("P√£gina Principal");
			function.SetName("main");
			function.SetNameUrl("main.jsp");
			function.AddProfile(admin);
			function.AddProfile(oper);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("P√£gina Sobre");
			function.SetName("main_sobre");
			function.SetNameUrl("main_sobre.jsp");
			function.AddProfile(admin);
			function.AddProfile(oper);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Inclus√£o de PMV");
			function.SetName("incluiPmv");
			function.SetNameUrl("pmv.do/create");
			function.AddProfile(admin);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Edi√£√£o de PMV");
			function.SetName("editaPmv");
			function.SetNameUrl("pmv.do/edit");
			function.AddProfile(admin);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Exclus√£o de Pmv");
			function.SetName("excluiPmv");
			function.SetNameUrl("pmv.do/delete");
			function.AddProfile(admin);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Consulta de PMV");
			function.SetName("listaPmv");
			function.SetNameUrl("pmv.do/view");
			function.AddProfile(admin);
			function.AddProfile(oper);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Controller do PMV");
			function.SetName("PMV");
			function.SetNameUrl("searchPmv.do/edit");
			function.AddProfile(admin);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Controller do PMV");
			function.SetName("PMV");
			function.SetNameUrl("searchPmv.do/delete");
			function.AddProfile(admin);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Controller do PMV");
			function.SetName("PMV");
			function.SetNameUrl("searchPmv.do/view");
			function.AddProfile(admin);
			function.AddProfile(oper);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Inclus√£o de Fornecedor");
			function.SetName("incluiFornecedor");
			function.SetNameUrl("constructor.do/create");
			function.AddProfile(admin);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Edi√£√£o de Fornecedor");
			function.SetName("editaFornecedor");
			function.SetNameUrl("constructor.do/edit");
			function.AddProfile(admin);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Exclus√£o de Fornecedor");
			function.SetName("excluiFornecedor");
			function.SetNameUrl("constructor.do/delete");
			function.AddProfile(admin);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Consulta de Fornecedor");
			function.SetName("listaFornecedor");
			function.SetNameUrl("constructor.do/view");
			function.AddProfile(admin);
			function.AddProfile(oper);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Controller do Fornecedor");
			function.SetName("Fornecedor");
			function.SetNameUrl("searchConstructor.do/edit");
			function.AddProfile(admin);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Controller do Fornecedor");
			function.SetName("Fornecedor");
			function.SetNameUrl("searchConstructor.do/delete");
			function.AddProfile(admin);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Controller do Fornecedor");
			function.SetName("Fornecedor");
			function.SetNameUrl("searchConstructor.do/view");
			function.AddProfile(admin);
			function.AddProfile(oper);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Inclus√£o de AT");
			function.SetName("incluiAT");
			function.SetNameUrl("at.do/create");
			function.AddProfile(admin);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Edi√£√£o de AT");
			function.SetName("editaAT");
			function.SetNameUrl("at.do/edit");
			function.AddProfile(admin);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Exclus√£o de AT");
			function.SetName("excluiAT");
			function.SetNameUrl("at.do/delete");
			function.AddProfile(admin);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Consulta de AT");
			function.SetName("listaAT");
			function.SetNameUrl("at.do/view");
			function.AddProfile(admin);
			function.AddProfile(oper);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Controller do AT");
			function.SetName("AT");
			function.SetNameUrl("searchAt.do/edit");
			function.AddProfile(admin);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Controller do AT");
			function.SetName("AT");
			function.SetNameUrl("searchAt.do/delete");
			function.AddProfile(admin);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Controller do AT");
			function.SetName("AT");
			function.SetNameUrl("searchAt.do/view");
			function.AddProfile(admin);
			function.AddProfile(oper);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Inclus√£o de Sensor AT");
			function.SetName("incluiSensorAT");
			function.SetNameUrl("sensorAt.do/create");
			function.AddProfile(admin);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Edi√£√£o de Sensor AT");
			function.SetName("editaSensorAT");
			function.SetNameUrl("sensorAt.do/edit");
			function.AddProfile(admin);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Exclus√£o de Sensor AT");
			function.SetName("excluiSensorAT");
			function.SetNameUrl("sensorAt.do/delete");
			function.AddProfile(admin);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Consulta de Sensor AT");
			function.SetName("listaSensorAT");
			function.SetNameUrl("sensorAt.do/view");
			function.AddProfile(admin);
			function.AddProfile(oper);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Controller do Sensor AT");
			function.SetName("SensorAT");
			function.SetNameUrl("searchSensorAt.do/edit");
			function.AddProfile(admin);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Controller do Sensor AT");
			function.SetName("SensorAT");
			function.SetNameUrl("searchSensorAt.do/delete");
			function.AddProfile(admin);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Controller do Sensor AT");
			function.SetName("SensorAT");
			function.SetNameUrl("searchSensorAt.do/view");
			function.AddProfile(admin);
			function.AddProfile(oper);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Inclus√£o de Meteo");
			function.SetName("incluiMeteo");
			function.SetNameUrl("meteo.do/create");
			function.AddProfile(admin);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Edi√£√£o de Meteo");
			function.SetName("editaMeteo");
			function.SetNameUrl("meteo.do/edit");
			function.AddProfile(admin);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Exclus√£o de Meteo");
			function.SetName("excluiMeteo");
			function.SetNameUrl("meteo.do/delete");
			function.AddProfile(admin);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Consulta de Meteo");
			function.SetName("listaMeteo");
			function.SetNameUrl("meteo.do/view");
			function.AddProfile(admin);
			function.AddProfile(oper);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Controller do Meteo");
			function.SetName("Meteo");
			function.SetNameUrl("searchMeteo.do/edit");
			function.AddProfile(admin);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Controller do Meteo");
			function.SetName("Meteo");
			function.SetNameUrl("searchMeteo.do/delete");
			function.AddProfile(admin);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Controller do Meteo");
			function.SetName("Meteo");
			function.SetNameUrl("searchMeteo.do/view");
			function.AddProfile(admin);
			function.AddProfile(oper);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Inclus√£o de Sensor Meteo");
			function.SetName("incluiSensorMeteo");
			function.SetNameUrl("sensorMeteo.do/create");
			function.AddProfile(admin);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Edi√£√£o de Sensor Meteo");
			function.SetName("editaSensorMeteo");
			function.SetNameUrl("sensorMeteo.do/edit");
			function.AddProfile(admin);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Exclus√£o de Sensor Meteo");
			function.SetName("excluiSensorMeteo");
			function.SetNameUrl("sensorMeteo.do/delete");
			function.AddProfile(admin);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Consulta de Sensor Meteo");
			function.SetName("listaSensorMeteo");
			function.SetNameUrl("sensorMeteo.do/view");
			function.AddProfile(admin);
			function.AddProfile(oper);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Controller do Sensor Meteo");
			function.SetName("SensorMeteo");
			function.SetNameUrl("searchSensorMeteo.do/edit");
			function.AddProfile(admin);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Controller do Sensor Meteo");
			function.SetName("SensorMeteo");
			function.SetNameUrl("searchSensorMeteo.do/delete");
			function.AddProfile(admin);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Controller do Sensor Meteo");
			function.SetName("SensorMeteo");
			function.SetNameUrl("searchSensorMeteo.do/view");
			function.AddProfile(admin);
			function.AddProfile(oper);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Controller do PmvMessage");
			function.SetName("sendPmvMessage");
			function.SetNameUrl("sendMessagePmv.do/edit");
			function.AddProfile(admin);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Controller do ActiveConf");
			function.SetName("activeConf");
			function.SetNameUrl("activeConf.do/edit");
			function.AddProfile(admin);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Controller do Monitor");
			function.SetName("monitor");
			function.SetNameUrl("monitor.do/view");
			function.AddProfile(admin);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Controller do Monitor");
			function.SetName("monitor");
			function.SetNameUrl("monitor.do/view");
			function.AddProfile(oper);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Controller da Consulta Mensage");
			function.SetName("searchMessagePMV");
			function.SetNameUrl("searchMessagePmv.do/view");
			function.AddProfile(admin);
			odb.Store(function);
			function = new NeoDatis.Odb.Test.Update.Nullobject.Functions();
			function.SetDescription("Controller da Consulta Mensage");
			function.SetName("searchMessagePMV");
			function.SetNameUrl("searchMessagePmv.do/view");
			function.AddProfile(oper);
			odb.Store(function);
		}
	}
}
