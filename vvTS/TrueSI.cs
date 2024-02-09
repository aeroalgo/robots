using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000061 RID: 97
	[HandlerCategory("vvIndicators"), HandlerDecimals(2), HandlerName("TSI")]
	public class TrueSI : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x0600036C RID: 876 RVA: 0x000136F8 File Offset: 0x000118F8
		public IList<double> Execute(IList<double> src)
		{
			int count = src.Count;
			IList<double> list = new double[count];
			IList<double> list2 = new double[count];
			IList<double> list3 = new double[count];
			for (int i = 1; i < count; i++)
			{
				list2[i] = src[i] - src[i - 1];
				list3[i] = Math.Abs(list2[i]);
			}
			IList<double> src2 = EMA.GenEMA(list2, this.First_R);
			IList<double> src3 = EMA.GenEMA(list3, this.First_R);
			IList<double> list4 = EMA.GenEMA(src2, this.Second_S);
			IList<double> list5 = EMA.GenEMA(src3, this.Second_S);
			for (int j = 0; j < count; j++)
			{
				if (j <= 1)
				{
					list[j] = 0.0;
				}
				else
				{
					list[j] = 100.0 * list4[j] / list5[j];
				}
			}
			return list;
		}

		// Token: 0x17000125 RID: 293
		public IContext Context
		{
			// Token: 0x0600036D RID: 877 RVA: 0x000137EE File Offset: 0x000119EE
			get;
			// Token: 0x0600036E RID: 878 RVA: 0x000137F6 File Offset: 0x000119F6
			set;
		}

		// Token: 0x17000123 RID: 291
		[HandlerParameter(true, "5", Min = "3", Max = "50", Step = "1")]
		public int First_R
		{
			// Token: 0x06000368 RID: 872 RVA: 0x000136D4 File Offset: 0x000118D4
			get;
			// Token: 0x06000369 RID: 873 RVA: 0x000136DC File Offset: 0x000118DC
			set;
		}

		// Token: 0x17000124 RID: 292
		[HandlerParameter(true, "8", Min = "6", Max = "20", Step = "1")]
		public int Second_S
		{
			// Token: 0x0600036A RID: 874 RVA: 0x000136E5 File Offset: 0x000118E5
			get;
			// Token: 0x0600036B RID: 875 RVA: 0x000136ED File Offset: 0x000118ED
			set;
		}
	}
}
