using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000079 RID: 121
	[HandlerCategory("vvWilliams"), HandlerName("Williams UO")]
	public class UltimateOscillator : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000457 RID: 1111 RVA: 0x00016D14 File Offset: 0x00014F14
		public IList<double> Execute(ISecurity _sec)
		{
			IList<double> lowPrices = _sec.get_LowPrices();
			IList<double> closePrices = _sec.get_ClosePrices();
			IList<Bar> Bars = _sec.get_Bars();
			IList<double> list = new List<double>(closePrices.Count);
			IList<double> va1 = new List<double>(closePrices.Count);
			double num = (double)(this.FastK + this.MiddleK + this.SlowK);
			for (int i = 0; i < closePrices.Count; i++)
			{
				double num2;
				if (i < 1)
				{
					num2 = 0.0;
				}
				else
				{
					num2 = Math.Min(lowPrices[i], closePrices[i - 1]);
				}
				double item = closePrices[i] - num2;
				va1.Add(item);
			}
			IList<double> data = this.Context.GetData("sma", new string[]
			{
				this.FastPeriod.ToString(),
				va1.GetHashCode().ToString()
			}, () => Series.SMA(va1, this.FastPeriod));
			IList<double> data2 = this.Context.GetData("sma", new string[]
			{
				this.MiddlePeriod.ToString(),
				va1.GetHashCode().ToString()
			}, () => Series.SMA(va1, this.MiddlePeriod));
			IList<double> data3 = this.Context.GetData("sma", new string[]
			{
				this.SlowPeriod.ToString(),
				va1.GetHashCode().ToString()
			}, () => Series.SMA(va1, this.SlowPeriod));
			IList<double> data4 = this.Context.GetData("atr", new string[]
			{
				this.FastPeriod.ToString(),
				_sec.get_CacheName()
			}, () => Series.AverageTrueRange(Bars, this.FastPeriod));
			IList<double> data5 = this.Context.GetData("atr", new string[]
			{
				this.MiddlePeriod.ToString(),
				_sec.get_CacheName()
			}, () => Series.AverageTrueRange(Bars, this.MiddlePeriod));
			IList<double> data6 = this.Context.GetData("atr", new string[]
			{
				this.SlowPeriod.ToString(),
				_sec.get_CacheName()
			}, () => Series.AverageTrueRange(Bars, this.SlowPeriod));
			int num3 = Math.Max(Math.Max(this.FastPeriod, this.MiddlePeriod), this.SlowPeriod);
			for (int j = 0; j < closePrices.Count; j++)
			{
				double item2;
				if (j < num3)
				{
					item2 = 0.0;
				}
				else
				{
					double num4 = (double)this.FastK * data[j] / data4[j] + (double)this.MiddleK * data2[j] / data5[j] + (double)this.SlowK * data3[j] / data6[j];
					item2 = num4 / num * 100.0;
				}
				list.Add(item2);
			}
			return list;
		}

		// Token: 0x1700017B RID: 379
		public IContext Context
		{
			// Token: 0x06000458 RID: 1112 RVA: 0x00017064 File Offset: 0x00015264
			get;
			// Token: 0x06000459 RID: 1113 RVA: 0x0001706C File Offset: 0x0001526C
			set;
		}

		// Token: 0x17000178 RID: 376
		[HandlerParameter(true, "4", Min = "1", Max = "20", Step = "1")]
		public int FastK
		{
			// Token: 0x06000451 RID: 1105 RVA: 0x00016C49 File Offset: 0x00014E49
			get;
			// Token: 0x06000452 RID: 1106 RVA: 0x00016C51 File Offset: 0x00014E51
			set;
		}

		// Token: 0x17000175 RID: 373
		[HandlerParameter(true, "7", Min = "1", Max = "100", Step = "1")]
		public int FastPeriod
		{
			// Token: 0x0600044B RID: 1099 RVA: 0x00016C16 File Offset: 0x00014E16
			get;
			// Token: 0x0600044C RID: 1100 RVA: 0x00016C1E File Offset: 0x00014E1E
			set;
		}

		// Token: 0x17000179 RID: 377
		[HandlerParameter(true, "2", Min = "1", Max = "20", Step = "1")]
		public int MiddleK
		{
			// Token: 0x06000453 RID: 1107 RVA: 0x00016C5A File Offset: 0x00014E5A
			get;
			// Token: 0x06000454 RID: 1108 RVA: 0x00016C62 File Offset: 0x00014E62
			set;
		}

		// Token: 0x17000176 RID: 374
		[HandlerParameter(true, "14", Min = "1", Max = "100", Step = "1")]
		public int MiddlePeriod
		{
			// Token: 0x0600044D RID: 1101 RVA: 0x00016C27 File Offset: 0x00014E27
			get;
			// Token: 0x0600044E RID: 1102 RVA: 0x00016C2F File Offset: 0x00014E2F
			set;
		}

		// Token: 0x1700017A RID: 378
		[HandlerParameter(true, "1", Min = "1", Max = "20", Step = "1")]
		public int SlowK
		{
			// Token: 0x06000455 RID: 1109 RVA: 0x00016C6B File Offset: 0x00014E6B
			get;
			// Token: 0x06000456 RID: 1110 RVA: 0x00016C73 File Offset: 0x00014E73
			set;
		}

		// Token: 0x17000177 RID: 375
		[HandlerParameter(true, "28", Min = "1", Max = "100", Step = "1")]
		public int SlowPeriod
		{
			// Token: 0x0600044F RID: 1103 RVA: 0x00016C38 File Offset: 0x00014E38
			get;
			// Token: 0x06000450 RID: 1104 RVA: 0x00016C40 File Offset: 0x00014E40
			set;
		}
	}
}
