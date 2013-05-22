/*
NeoDatis ODB : Native Object Database (odb.info@NeoDatis.org)
Copyright (C) 2008 NeoDatis Inc. http://www.NeoDatis.org

"This file is part of the NeoDatis ODB open source object database".

NeoDatis ODB is free software; you can redistribute it and/or
modify it under the terms of the GNU Lesser General Public
License as published by the Free Software Foundation; either
version 2.1 of the License, or (at your option) any later version.

NeoDatis ODB is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
Lesser General Public License for more details.

You should have received a copy of the GNU Lesser General Public
License along with this library; if not, write to the Free Software
Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
*/
using System;
using NeoDatis.Odb;
using NeoDatis.Odb.Tool;
//using NUnit.Framework;
using NeoDatis.Tool;
using NeoDatis.Odb.Impl.Core.Server.Layers.Layer3.Engine;
using NeoDatis.Odb.Core.Server.Message;
using NeoDatis.Odb.Core;
using NUnit.Framework;
namespace NeoDatis.Odb.Test
{
	//[TestFixture]
	public class ODBTest:NeoDatisAssert
	{
		public static bool isLocal = true;
		public static System.String HOST = "localhost";
		public static int PORT = 10000;
        public static string Directory = ".";
		public static bool runAll = false;
        public static bool useSameVmOptimization = false;
        public static bool testNewFeature = false;
        public static bool testPerformance = false;

        public string GetBaseName()
        {
            return GetName()+DateTime.Now.Ticks+".neodatis";
        }


        public virtual ODB Open(System.String fileName, System.String user, System.String password)
		{
			if (isLocal)
			{
				return ODBFactory.Open(fileName, user, password);
			}
			return ODBFactory.OpenClient(HOST, PORT, fileName, user, password);
		}

        public virtual ODB Open(System.String fileName)
		{
			if (isLocal)
			{
				return ODBFactory.Open(fileName, null, null);
			}
			return ODBFactory.OpenClient(HOST, 10000, fileName);
		}

        public virtual ODB OpenLocal(System.String fileName)
		{
			return ODBFactory.Open(fileName, null, null);
		}
		
		public virtual ODBServer OpenServer(int port)
		{
			return ODBFactory.OpenServer(port);
		}

        public virtual ODB OpenClient(System.String host, int port, System.String baseIdentifier)
		{
			return ODBFactory.OpenClient(host, port, baseIdentifier);
		}
		
		public virtual void  failCS()
		{
			AssertTrue(true);
		}
		
		protected internal virtual void  FailNotImplemented(System.String string_Renamed)
		{
			AssertTrue(true);
		}
		
		protected internal virtual void  DeleteBase(System.String baseName)
		{
			if (isLocal)
			{
				IOUtil.DeleteFile(baseName);
			}
			else
			{
				ServerAdmin sa = new ServerAdmin(HOST, PORT);
				DeleteBaseMessage message = new DeleteBaseMessage(baseName);
				DeleteBaseMessageResponse rmessage = (DeleteBaseMessageResponse) sa.sendMessage(message);
				if (rmessage.HasError())
				{
					throw new ODBRuntimeException(NeoDatisError.InternalError.AddParameter(rmessage.GetError()));
				}
			}
		}
		public virtual void  t1estzzzz()
		{
			
		}
		public void Println(object o)
		{
            System.Console.WriteLine(o.ToString());		   
		}
	}
}