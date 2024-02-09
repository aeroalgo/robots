using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000104 RID: 260
	[HandlerCategory("vvTrade"), HandlerName("Открытие сессии")]
	public class SessionOpen : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs
	{
		// Token: 0x06000772 RID: 1906 RVA: 0x00020B3C File Offset: 0x0001ED3C
		public IList<double> Execute(ISecurity sec)
		{
			IList<Bar> bars = sec.get_Bars();
			double[] array = new double[bars.Count];
			if (array.Length > 0)
			{
				List<double> list = new List<double>();
				for (int i = 0; i <= this.Session; i++)
				{
					list.Add(bars[0].get_Open());
				}
				for (int j = 1; j < array.Length; j++)
				{
					if (bars[j - 1].get_Date().Day != bars[j].get_Date().Day)
					{
						list.RemoveAt(0);
						list.Add(bars[j].get_Open());
					}
					array[j] = list[0];
				}
			}
			return array;
		}

		// Token: 0x17000262 RID: 610
		[HandlerParameter(true, "1", Min = "0", Max = "10", Step = "1")]
		public int Session
		{
			// Token: 0x06000770 RID: 1904 RVA: 0x00020B28 File Offset: 0x0001ED28
			get;
			// Token: 0x06000771 RID: 1905 RVA: 0x00020B30 File Offset: 0x0001ED30
			set;
		}
	}
}
