using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x0200006A RID: 106
	[HandlerCategory("vvIndicators"), HandlerName("VHF (Vertical Horizontal Filter)")]
	public class VHF : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x060003B1 RID: 945 RVA: 0x000148E8 File Offset: 0x00012AE8
		public IList<double> Execute(ISecurity sec)
		{
			return this.Context.GetData("VHF", new string[]
			{
				this.N.ToString(),
				sec.get_CacheName()
			}, () => VHF.GenVHF(sec, this.N, this.Context));
		}

		// Token: 0x060003B0 RID: 944 RVA: 0x0001472C File Offset: 0x0001292C
		public static IList<double> GenVHF(ISecurity sec, int _N, IContext ctx)
		{
			if (_N <= 1)
			{
				_N = 10;
			}
			int count = sec.get_Bars().Count;
			if (count < _N)
			{
				return null;
			}
			IList<double> closePrices = sec.get_ClosePrices();
			double[] array = new double[count];
			double[] array2 = new double[count];
			for (int i = 1; i < count; i++)
			{
				array2[i] = Math.Abs(closePrices[i] - closePrices[i - 1]);
			}
			IList<double> data = ctx.GetData("hhv", new string[]
			{
				_N.ToString(),
				sec.get_CacheName()
			}, () => Series.Highest(sec.get_HighPrices(), _N));
			IList<double> data2 = ctx.GetData("llv", new string[]
			{
				_N.ToString(),
				sec.get_CacheName()
			}, () => Series.Lowest(sec.get_LowPrices(), _N));
			for (int j = _N; j < count; j++)
			{
				double num = data[j];
				double num2 = data2[j];
				double num3 = num - num2;
				double num4 = 0.0;
				for (int k = j - _N + 1; k <= j; k++)
				{
					num4 += array2[k];
				}
				array[j] = num3 / num4;
			}
			return array;
		}

		// Token: 0x1700013D RID: 317
		public IContext Context
		{
			// Token: 0x060003B2 RID: 946 RVA: 0x0001494C File Offset: 0x00012B4C
			get;
			// Token: 0x060003B3 RID: 947 RVA: 0x00014954 File Offset: 0x00012B54
			set;
		}

		// Token: 0x1700013C RID: 316
		[HandlerParameter(true, "28", Min = "10", Max = "30", Step = "1")]
		public int N
		{
			// Token: 0x060003AE RID: 942 RVA: 0x000146E2 File Offset: 0x000128E2
			get;
			// Token: 0x060003AF RID: 943 RVA: 0x000146EA File Offset: 0x000128EA
			set;
		}
	}
}
