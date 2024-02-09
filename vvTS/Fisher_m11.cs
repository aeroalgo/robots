using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x0200002B RID: 43
	[HandlerCategory("vvIndicators"), HandlerName("Fisher m11")]
	public class Fisher_m11 : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x0600018A RID: 394 RVA: 0x0000781C File Offset: 0x00005A1C
		public IList<double> Execute(ISecurity sec)
		{
			return this.Context.GetData("FisherM11", new string[]
			{
				this.Period.ToString(),
				this.PriceSmoothing.ToString(),
				this.IndexSmoothing.ToString(),
				sec.get_CacheName()
			}, () => Fisher_m11.GenFisher_m11(sec, this.Period, this.PriceSmoothing, this.IndexSmoothing, this.Context));
		}

		// Token: 0x06000189 RID: 393 RVA: 0x00007490 File Offset: 0x00005690
		public static IList<double> GenFisher_m11(ISecurity _sec, int _period, double _PriceSmoothing, double _IndexSmoothing, IContext Context)
		{
			IList<double> list = new List<double>(_sec.get_ClosePrices().Count);
			IList<double> list2 = new List<double>(_sec.get_ClosePrices().Count);
			IList<double> mids = Context.GetData("MedianPrice", new string[]
			{
				_sec.get_CacheName()
			}, () => Series.MedianPrice(_sec.get_Bars()));
			IList<double> data = Context.GetData("medianprice_hhv", new string[]
			{
				_period.ToString(),
				mids.GetHashCode().ToString()
			}, () => Series.Highest(mids, _period));
			IList<double> data2 = Context.GetData("medianprice_llv", new string[]
			{
				_period.ToString(),
				mids.GetHashCode().ToString()
			}, () => Series.Lowest(mids, _period));
			double num = 0.0;
			double num2 = 0.0;
			double tick = _sec.get_Tick();
			if (_PriceSmoothing >= 1.0)
			{
				_PriceSmoothing = 0.9999;
			}
			if (_PriceSmoothing < 0.0)
			{
				_PriceSmoothing = 0.0;
			}
			if (_IndexSmoothing >= 1.0)
			{
				_IndexSmoothing = 0.9999;
			}
			if (_IndexSmoothing < 0.0)
			{
				_IndexSmoothing = 0.0;
			}
			for (int i = 0; i < _period; i++)
			{
				list.Add(0.0);
				list2.Add(0.0);
			}
			for (int j = _period; j < _sec.get_Bars().Count; j++)
			{
				double num3 = data[j];
				double num4 = data2[j];
				if (num3 - num4 < 0.1 * tick)
				{
					num3 = num4 + 0.1 * tick;
				}
				double num5 = num3 - num4;
				double num6 = (_sec.get_HighPrices()[j] + _sec.get_LowPrices()[j]) / 2.0;
				if (num5 != 0.0)
				{
					num = 2.0 * (num6 - num4) / num5 - 1.0;
				}
				double num7 = _PriceSmoothing * list2[j - 1] + (1.0 - _PriceSmoothing) * num;
				list2.Add(num7);
				double num8 = num7;
				num8 = Math.Min(Math.Max(num8, -0.999), 0.999);
				if (1.0 - num8 != 0.0)
				{
					num2 = Math.Log((1.0 + num8) / (1.0 - num8));
				}
				num7 = _IndexSmoothing * list[j - 1] + (1.0 - _IndexSmoothing) * num2;
				list.Add(num7);
			}
			return list;
		}

		// Token: 0x17000083 RID: 131
		public IContext Context
		{
			// Token: 0x0600018B RID: 395 RVA: 0x000078A3 File Offset: 0x00005AA3
			get;
			// Token: 0x0600018C RID: 396 RVA: 0x000078AB File Offset: 0x00005AAB
			set;
		}

		// Token: 0x17000082 RID: 130
		[HandlerParameter(true, "0.3", Min = "0", Max = "0.9999", Step = "0.1")]
		public double IndexSmoothing
		{
			// Token: 0x06000187 RID: 391 RVA: 0x0000743C File Offset: 0x0000563C
			get;
			// Token: 0x06000188 RID: 392 RVA: 0x00007444 File Offset: 0x00005644
			set;
		}

		// Token: 0x17000080 RID: 128
		[HandlerParameter(true, "10", Min = "1", Max = "100", Step = "1")]
		public int Period
		{
			// Token: 0x06000183 RID: 387 RVA: 0x0000741A File Offset: 0x0000561A
			get;
			// Token: 0x06000184 RID: 388 RVA: 0x00007422 File Offset: 0x00005622
			set;
		}

		// Token: 0x17000081 RID: 129
		[HandlerParameter(true, "0.3", Min = "0", Max = "0.9999", Step = "0.1")]
		public double PriceSmoothing
		{
			// Token: 0x06000185 RID: 389 RVA: 0x0000742B File Offset: 0x0000562B
			get;
			// Token: 0x06000186 RID: 390 RVA: 0x00007433 File Offset: 0x00005633
			set;
		}
	}
}
