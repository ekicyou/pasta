using NeoDatis.Odb.Test.VO.Download;
using NeoDatis.Odb.Impl.Core.Query.Criteria;
using NeoDatis.Odb.Core.Query.Criteria;
using NUnit.Framework;
namespace NeoDatis.Odb.Test.Other
{
	[TestFixture]
    public class TestDownloadManager : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		public virtual void NewDownload(string name, string email, string downloadType, string
			 fileName)
		{
			NeoDatis.Odb.ODB odb = null;
			User user = null;
			try
			{
				odb = Open("download.neodatis");
				NeoDatis.Odb.Objects<User> users = odb.GetObjects<User>(new CriteriaQuery(Where.Equal("email", email)));
				if (users.Count!=0)
				{
					user = (User)users.GetFirst();
					user.SetLastDownload(new System.DateTime());
					user.SetNbDownloads(user.GetNbDownloads() + 1);
					odb.Store(user);
				}
				else
				{
					user = new User();
					user.SetName(name);
					user.SetEmail(email);
					user.SetLastDownload(new System.DateTime());
					user.SetNbDownloads(1);
					odb.Store(user);
				}
				NeoDatis.Odb.Test.VO.Download.Download download = new NeoDatis.Odb.Test.VO.Download.Download
					();
				download.SetFileName(fileName);
				download.SetType(downloadType);
				download.SetUser(user);
				download.SetWhen(new System.DateTime());
				odb.Store(download);
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
        public virtual void Test1()
		{
			NeoDatis.Odb.Test.Other.TestDownloadManager tdm = new NeoDatis.Odb.Test.Other.TestDownloadManager
				();
			tdm.NewDownload("olivier", "olivier@neodatis.com", "knowledger", "knowledger1.1");
			tdm.NewDownload("olivier", "olivier@neodatis.com", "knowledger", "knowledger1.1");
			NeoDatis.Odb.ODB odb = Open("download.neodatis");
			AssertEquals(2, odb.Count(new CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Download.Download))));
			AssertEquals(1, odb.Count(new CriteriaQuery
				(typeof(User))));
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test2()
		{
			NeoDatis.Odb.Test.Other.TestDownloadManager tdm = new NeoDatis.Odb.Test.Other.TestDownloadManager
				();
			int size = (isLocal ? 1000 : 50);
			for (int i = 0; i < size; i++)
			{
				tdm.NewDownload("olivier", "olivier@neodatis.com", "knowledger", "knowledger1.1");
				tdm.NewDownload("olivier", "olivier@neodatis.com", "knowledger", "knowledger1.1");
			}
			NeoDatis.Odb.ODB odb = Open("download.neodatis");
			AssertEquals(size * 2, odb.Count(new CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Download.Download))));
			AssertEquals(1, odb.Count(new CriteriaQuery
				(typeof(User))));
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public override void SetUp()
		{
			base.SetUp();
			DeleteBase("download.neodatis");
		}

		/// <exception cref="System.Exception"></exception>
		public override void TearDown()
		{
			DeleteBase("download.neodatis");
		}

		/// <exception cref="System.Exception"></exception>
		public static void Main2(string[] args)
		{
			NeoDatis.Odb.Test.Other.TestDownloadManager td = new NeoDatis.Odb.Test.Other.TestDownloadManager
				();
			for (int i = 0; i < 2000; i++)
			{
				td.SetUp();
				td.Test1();
				td.TearDown();
				td.Test1();
			}
		}
	}
}
