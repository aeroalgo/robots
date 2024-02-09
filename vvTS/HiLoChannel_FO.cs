using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000120 RID: 288
	[HandlerCategory("vvBands&Channels"), HandlerName("HiLoChannel_FO")]
	public class HiLoChannel_FO : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000839 RID: 2105 RVA: 0x0002312C File Offset: 0x0002132C
		public IList<double> Execute(ISecurity sec)
		{
			return this.Context.GetData("HiLoChannel_FO", new string[]
			{
				this.NBars.ToString(),
				this.MAperiod.ToString(),
				sec.get_CacheName()
			}, () => HiLoChannel_FO.GenHiLoChannel_FO(sec, this.Context, this.NBars, this.MAperiod));
		}

		// Token: 0x06000838 RID: 2104 RVA: 0x00022F18 File Offset: 0x00021118
		public static IList<double> GenHiLoChannel_FO(ISecurity src, IContext ctx, int _nbars, int _maperiod)
		{
			int count = src.get_Bars().Count;
			double[] array = new double[count];
			double[] array2 = new double[count];
			IList<double> data = ctx.GetData("hhv", new string[]
			{
				_nbars.ToString(),
				src.get_CacheName()
			}, () => Series.Highest(src.get_HighPrices(), _nbars));
			IList<double> data2 = ctx.GetData("llv", new string[]
			{
				_nbars.ToString(),
				src.get_CacheName()
			}, () => Series.Lowest(src.get_LowPrices(), _nbars));
			for (int i = 0; i < count; i++)
			{
				double num = data[i];
				double num2 = data2[i];
				double num3;
				if (num > num2)
				{
					num3 = 100.0 * (src.get_ClosePrices()[i] - num2) / (num - num2);
				}
				else
				{
					num3 = 0.0;
				}
				if (num3 < 0.0)
				{
					num3 = 0.1;
				}
				if (num3 > 100.0)
				{
					num3 = 99.9;
				}
				array[i] = 0.1 * (num3 - 50.0);
			}
			IList<double> list = LWMA.GenWMA(array, _maperiod);
			for (int j = 0; j < count; j++)
			{
				array2[j] = (Math.Exp(2.0 * list[j]) - 1.0) / (Math.Exp(2.0 * list[j]) + 1.0);
			}
			return array2;
		}

		// Token: 0x1700029C RID: 668
		public IContext Context
		{
			// Token: 0x0600083A RID: 2106 RVA: 0x000231A1 File Offset: 0x000213A1
			get;
			// Token: 0x0600083B RID: 2107 RVA: 0x000231A9 File Offset: 0x000213A9
			set;
		}

		// Token: 0x1700029B RID: 667
		[HandlerParameter(true, "9", Min = "1", Max = "50", Step = "1")]
		public int MAperiod
		{
			// Token: 0x06000836 RID: 2102 RVA: 0x00022ECD File Offset: 0x000210CD
			get;
			// Token: 0x06000837 RID: 2103 RVA: 0x00022ED5 File Offset: 0x000210D5
			set;
		}

		// Token: 0x1700029A RID: 666
		[HandlerParameter(true, "50", Min = "1", Max = "100", Step = "1")]
		public int NBars
		{
			// Token: 0x06000834 RID: 2100 RVA: 0x00022EBC File Offset: 0x000210BC
			get;
			// Token: 0x06000835 RID: 2101 RVA: 0x00022EC4 File Offset: 0x000210C4
			set;
		}
	}
}
