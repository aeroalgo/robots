using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x0200001E RID: 30
	[HandlerCategory("vvIndicators"), HandlerName("CRS (CCI-RSI-Stoch)")]
	public class CRS : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000110 RID: 272 RVA: 0x0000579A File Offset: 0x0000399A
		public IList<double> Execute(ISecurity sec)
		{
			return this.GenCRS(sec);
		}

		// Token: 0x0600010F RID: 271 RVA: 0x0000553C File Offset: 0x0000373C
		public IList<double> GenCRS(ISecurity sec)
		{
			IList<double> closePrices = sec.get_ClosePrices();
			IList<double> High = sec.get_HighPrices();
			IList<double> Low = sec.get_LowPrices();
			int count = closePrices.Count;
			double[] array = new double[closePrices.Count];
			double[] array2 = new double[closePrices.Count];
			double[] array3 = new double[closePrices.Count];
			IList<double> data = this.Context.GetData("hhv", new string[]
			{
				this.StochK_period.ToString(),
				sec.get_CacheName()
			}, () => Series.Highest(High, this.StochK_period));
			IList<double> data2 = this.Context.GetData("llv", new string[]
			{
				this.StochK_period.ToString(),
				sec.get_CacheName()
			}, () => Series.Lowest(Low, this.StochK_period));
			for (int i = 0; i < count; i++)
			{
				double num = data[i] - data2[i];
				array2[i] = ((num == 0.0) ? 0.0 : (100.0 * (closePrices[i] - Low[i]) / num));
			}
			IList<double> list = Series.RSI(closePrices, this.RSIperiod);
			IList<double> list2 = Series.CCI(sec.get_Bars(), this.CCIperiod);
			for (int j = 0; j < count; j++)
			{
				array3[j] = 100.0 * (closePrices[j] - data2[j]) / (data[j] - data2[j]) - 50.0;
				IList<double> list3;
				int index;
				(list3 = list)[index = j] = list3[index] - 50.0;
				array[j] = array3[j] * (1.0 - this.CCIweight - this.RSIweight) + list[j] * this.RSIweight + list2[j] * this.CCIweight;
			}
			IList<double> result = LWMA.GenWMA(array, this.FastMA);
			IList<double> result2 = LWMA.GenWMA(array, this.SlowMA);
			if (!this.SignalLine)
			{
				return result;
			}
			return result2;
		}

		// Token: 0x17000055 RID: 85
		[HandlerParameter(true, "21", Min = "1", Max = "90", Step = "1")]
		public int CCIperiod
		{
			// Token: 0x06000103 RID: 259 RVA: 0x0000549E File Offset: 0x0000369E
			get;
			// Token: 0x06000104 RID: 260 RVA: 0x000054A6 File Offset: 0x000036A6
			set;
		}

		// Token: 0x17000053 RID: 83
		[HandlerParameter(true, "0.1", Min = "0", Max = "1", Step = "0.1")]
		public double CCIweight
		{
			// Token: 0x060000FF RID: 255 RVA: 0x0000547C File Offset: 0x0000367C
			get;
			// Token: 0x06000100 RID: 256 RVA: 0x00005484 File Offset: 0x00003684
			set;
		}

		// Token: 0x1700005B RID: 91
		public IContext Context
		{
			// Token: 0x06000111 RID: 273 RVA: 0x000057A3 File Offset: 0x000039A3
			get;
			// Token: 0x06000112 RID: 274 RVA: 0x000057AB File Offset: 0x000039AB
			set;
		}

		// Token: 0x17000058 RID: 88
		[HandlerParameter(true, "10", Min = "1", Max = "50", Step = "1")]
		public int FastMA
		{
			// Token: 0x06000109 RID: 265 RVA: 0x000054D1 File Offset: 0x000036D1
			get;
			// Token: 0x0600010A RID: 266 RVA: 0x000054D9 File Offset: 0x000036D9
			set;
		}

		// Token: 0x17000056 RID: 86
		[HandlerParameter(true, "21", Min = "1", Max = "30", Step = "1")]
		public int RSIperiod
		{
			// Token: 0x06000105 RID: 261 RVA: 0x000054AF File Offset: 0x000036AF
			get;
			// Token: 0x06000106 RID: 262 RVA: 0x000054B7 File Offset: 0x000036B7
			set;
		}

		// Token: 0x17000054 RID: 84
		[HandlerParameter(true, "0.1", Min = "0", Max = "1", Step = "0.1")]
		public double RSIweight
		{
			// Token: 0x06000101 RID: 257 RVA: 0x0000548D File Offset: 0x0000368D
			get;
			// Token: 0x06000102 RID: 258 RVA: 0x00005495 File Offset: 0x00003695
			set;
		}

		// Token: 0x1700005A RID: 90
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool SignalLine
		{
			// Token: 0x0600010D RID: 269 RVA: 0x000054F3 File Offset: 0x000036F3
			get;
			// Token: 0x0600010E RID: 270 RVA: 0x000054FB File Offset: 0x000036FB
			set;
		}

		// Token: 0x17000059 RID: 89
		[HandlerParameter(true, "7", Min = "1", Max = "50", Step = "1")]
		public int SlowMA
		{
			// Token: 0x0600010B RID: 267 RVA: 0x000054E2 File Offset: 0x000036E2
			get;
			// Token: 0x0600010C RID: 268 RVA: 0x000054EA File Offset: 0x000036EA
			set;
		}

		// Token: 0x17000057 RID: 87
		[HandlerParameter(true, "24", Min = "1", Max = "50", Step = "1")]
		public int StochK_period
		{
			// Token: 0x06000107 RID: 263 RVA: 0x000054C0 File Offset: 0x000036C0
			get;
			// Token: 0x06000108 RID: 264 RVA: 0x000054C8 File Offset: 0x000036C8
			set;
		}
	}
}
