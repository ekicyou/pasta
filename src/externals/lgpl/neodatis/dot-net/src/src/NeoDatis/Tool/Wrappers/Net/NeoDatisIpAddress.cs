using System;
using System.Net;
namespace NeoDatis.Tool.Wrappers.Net{

	// A simple wrapper to get ip address
	public class NeoDatisIpAddress {
		public static string Get(string hostName){
			IPAddress a = Dns.GetHostAddresses(hostName)[0];
			return a.ToString();
		}
	}
}