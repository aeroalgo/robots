using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000105 RID: 261
	[HandlerCategory("vvTrade"), HandlerName("Закрытие сессии")]
	public class SessionClose : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs
	{
		// Token: 0x06000777 RID: 1911 RVA: 0x00020CD9 File Offset: 0x0001EED9
		public IList<double> Execute(ISecurity src)
		{
			return SessionClose.GetSessionClose(src, this.Session);
		}

		// Token: 0x06000776 RID: 1910 RVA: 0x00020C10 File Offset: 0x0001EE10
		public static IList<double> GetSessionClose(ISecurity src, int _Session)
		{
			IList<Bar> bars = src.get_Bars();
			double[] array = new double[bars.Count];
			if (array.Length > 0)
			{
				List<double> list = new List<double>();
				for (int i = 0; i <= _Session; i++)
				{
					list.Add(bars[0].get_Open());
				}
				for (int j = 1; j < array.Length; j++)
				{
					if (bars[j - 1].get_Date().Day != bars[j].get_Date().Day)
					{
						list.RemoveAt(0);
						list.Add(bars[j].get_Close());
					}
					list[_Session] = bars[j].get_Close();
					array[j] = list[0];
				}
			}
			return array;
		}

		// Token: 0x17000263 RID: 611
		[HandlerParameter(true, "1", Min = "0", Max = "10", Step = "1")]
		public int Session
		{
			// Token: 0x06000774 RID: 1908 RVA: 0x00020BFE File Offset: 0x0001EDFE
			get;
			// Token: 0x06000775 RID: 1909 RVA: 0x00020C06 File Offset: 0x0001EE06
			set;
		}
	}
}
