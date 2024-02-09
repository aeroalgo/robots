using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000193 RID: 403
	[HandlerCategory("vvAverages"), HandlerName("Optimal Tracking Filter")]
	public class OTFilter : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000CC8 RID: 3272 RVA: 0x000379BC File Offset: 0x00035BBC
		public IList<double> Execute(ISecurity sec)
		{
			return this.Context.GetData("OptimalTrackingFilter", new string[]
			{
				this.Period.ToString(),
				this.SlowingPeriod.ToString(),
				sec.get_CacheName()
			}, () => OTFilter.GenOTF(sec, this.Period, this.SlowingPeriod, this.Context));
		}

		// Token: 0x06000CC7 RID: 3271 RVA: 0x0003768C File Offset: 0x0003588C
		public static IList<double> GenOTF(ISecurity sec, int period, int slowingperiod, IContext context)
		{
			int count = sec.get_Bars().Count;
			IList<double> High = sec.get_HighPrices();
			IList<double> Low = sec.get_LowPrices();
			IList<double> arg_61_0 = sec.get_ClosePrices();
			IList<double> data = context.GetData("MedianPrice", new string[]
			{
				sec.get_CacheName()
			}, () => Series.MedianPrice(sec.get_Bars()));
			double[] array = new double[count];
			IList<double> list = data;
			if (slowingperiod > 0)
			{
				IList<double> data2 = context.GetData("hhv", new string[]
				{
					sec.get_CacheName(),
					slowingperiod.ToString()
				}, () => Series.Highest(High, slowingperiod));
				IList<double> data3 = context.GetData("llv", new string[]
				{
					sec.get_CacheName(),
					slowingperiod.ToString()
				}, () => Series.Lowest(Low, slowingperiod));
				for (int i = 0; i < count; i++)
				{
					array[i] = (data2[i] + data3[i]) / 2.0;
				}
				list = array;
				High = data2;
				Low = data3;
			}
			double[] array2 = new double[count];
			double[] array3 = new double[count];
			double[] array4 = new double[count];
			double num = 2.0 / (double)(period + 1);
			array2[0] = (array3[0] = (array4[0] = list[0]));
			for (int j = 1; j < count; j++)
			{
				array2[j] = num * (list[j] - list[j - 1]) + (1.0 - num) * array2[j - 1];
				array3[j] = num * (High[j] - Low[j]) / 2.0 + (1.0 - num) * array3[j - 1];
			}
			double num2 = 0.0;
			for (int k = 1; k < count; k++)
			{
				if (array3[k] != 0.0)
				{
					num2 = Math.Abs(array2[k] / array3[k]);
				}
				double num3 = (-num2 * num2 + num2 * Math.Sqrt(num2 * num2 + 16.0)) / 8.0;
				array4[k] = num3 * list[k] + (1.0 - num3) * array4[k - 1];
			}
			return array4;
		}

		// Token: 0x1700042E RID: 1070
		public IContext Context
		{
			// Token: 0x06000CC9 RID: 3273 RVA: 0x00037A31 File Offset: 0x00035C31
			get;
			// Token: 0x06000CCA RID: 3274 RVA: 0x00037A39 File Offset: 0x00035C39
			set;
		}

		// Token: 0x1700042C RID: 1068
		[HandlerParameter(true, "12", Min = "5", Max = "40", Step = "1")]
		public int Period
		{
			// Token: 0x06000CC3 RID: 3267 RVA: 0x00037627 File Offset: 0x00035827
			get;
			// Token: 0x06000CC4 RID: 3268 RVA: 0x0003762F File Offset: 0x0003582F
			set;
		}

		// Token: 0x1700042D RID: 1069
		[HandlerParameter(true, "0", Min = "1", Max = "20", Step = "1")]
		public int SlowingPeriod
		{
			// Token: 0x06000CC5 RID: 3269 RVA: 0x00037638 File Offset: 0x00035838
			get;
			// Token: 0x06000CC6 RID: 3270 RVA: 0x00037640 File Offset: 0x00035840
			set;
		}
	}
}
