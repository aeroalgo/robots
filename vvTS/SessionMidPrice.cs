using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200010C RID: 268
	[HandlerCategory("vvTrade"), HandlerName("Средняя цена за N сессий")]
	public class SessionMidPrice : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs
	{
		// Token: 0x0600078D RID: 1933 RVA: 0x0002119C File Offset: 0x0001F39C
		public IList<double> Execute(ISecurity src)
		{
			IList<Bar> bars = src.get_Bars();
			double[] array = new double[bars.Count];
			double[] array2 = new double[bars.Count];
			double[] array3 = new double[bars.Count];
			if (array2.Length > 0)
			{
				List<double> list = new List<double>();
				List<double> list2 = new List<double>();
				for (int i = 0; i <= this.Sessions; i++)
				{
					list.Add(bars[0].get_Low());
					list2.Add(bars[0].get_High());
				}
				for (int j = 1; j < array2.Length; j++)
				{
					if (bars[j - 1].get_Date().Day != bars[j].get_Date().Day)
					{
						list.RemoveAt(0);
						list.Add(bars[j].get_Low());
						list2.RemoveAt(0);
						list2.Add(bars[j].get_High());
					}
					list[this.Sessions] = Math.Min(list[this.Sessions], bars[j].get_Low());
					array2[j] = list[0];
					list2[this.Sessions] = Math.Max(list2[this.Sessions], bars[j].get_High());
					array3[j] = list2[0];
					array[j] = (array2[j] + array3[j]) / 2.0;
				}
			}
			return array;
		}

		// Token: 0x17000266 RID: 614
		[HandlerParameter(true, "1", Min = "0", Max = "10", Step = "1")]
		public int Sessions
		{
			// Token: 0x0600078B RID: 1931 RVA: 0x0002118A File Offset: 0x0001F38A
			get;
			// Token: 0x0600078C RID: 1932 RVA: 0x00021192 File Offset: 0x0001F392
			set;
		}
	}
}
