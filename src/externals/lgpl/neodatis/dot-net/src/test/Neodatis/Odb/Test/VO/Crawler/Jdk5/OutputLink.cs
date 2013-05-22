using NUnit.Framework;
namespace NeoDatis.Odb.Test.VO.Crawler.Jdk5
{
	public class OutputLink
	{
		private long id;

		private System.DateTime date;

		private string url;

		public OutputLink() : base()
		{
		}

		public OutputLink(long id, System.DateTime date, string url) : base()
		{
			// TODO Auto-generated constructor stub
			// TODO Auto-generated constructor stub
			this.id = id;
			this.date = date;
			this.url = url;
		}

		public virtual System.DateTime GetDate()
		{
			return date;
		}

		public virtual void SetDate(System.DateTime date)
		{
			this.date = date;
		}

		public virtual long GetId()
		{
			return id;
		}

		public virtual void SetId(long id)
		{
			this.id = id;
		}

		public virtual string GetUrl()
		{
			return url;
		}

		public virtual void SetUrl(string url)
		{
			this.url = url;
		}
	}
}
