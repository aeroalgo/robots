using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000181 RID: 385
	[HandlerCategory("vvAverages"), HandlerName("LRMA(LSMA)")]
	public class LRMA : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000C31 RID: 3121 RVA: 0x00034E24 File Offset: 0x00033024
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("lrma", new string[]
			{
				this.Period.ToString(),
				src.GetHashCode().ToString()
			}, () => LRMA.GenLRMA(src, this.Period));
		}

		// Token: 0x06000C2F RID: 3119 RVA: 0x00034D24 File Offset: 0x00032F24
		public static IList<double> GenLRMA(IList<double> src, int lrmaperiod)
		{
			IList<double> list = SMA.GenSMA(src, lrmaperiod);
			IList<double> list2 = LWMA.GenWMA(src, lrmaperiod);
			double[] array = new double[src.Count];
			for (int i = 0; i <= lrmaperiod; i++)
			{
				array[i] = list2[i];
			}
			for (int j = lrmaperiod; j < src.Count; j++)
			{
				array[j] = 3.0 * list2[j] - 2.0 * list[j];
			}
			return array;
		}

		// Token: 0x06000C30 RID: 3120 RVA: 0x00034DA4 File Offset: 0x00032FA4
		public static double iLSMA(IList<double> price, int period, int barNum)
		{
			if (barNum < period)
			{
				period = barNum;
			}
			double num = 0.0;
			for (int i = period; i >= 1; i--)
			{
				num += ((double)i - (double)(period + 1) / 3.0) * price[barNum - period + i];
			}
			return num * 6.0 / (double)(period * (period + 1));
		}

		// Token: 0x170003FF RID: 1023
		public IContext Context
		{
			// Token: 0x06000C2D RID: 3117 RVA: 0x00034D12 File Offset: 0x00032F12
			get;
			// Token: 0x06000C2E RID: 3118 RVA: 0x00034D1A File Offset: 0x00032F1A
			set;
		}

		// Token: 0x170003FE RID: 1022
		[HandlerParameter(true, "10", Min = "1", Max = "100", Step = "1")]
		public int Period
		{
			// Token: 0x06000C2B RID: 3115 RVA: 0x00034D01 File Offset: 0x00032F01
			get;
			// Token: 0x06000C2C RID: 3116 RVA: 0x00034D09 File Offset: 0x00032F09
			set;
		}
	}
}
