using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x0200002A RID: 42
	[HandlerCategory("vvIndicators"), HandlerName("Fisher Transform")]
	public class FisherTransform : BasePeriodIndicatorHandler, IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x0600017F RID: 383 RVA: 0x0000738C File Offset: 0x0000558C
		public IList<double> Execute(ISecurity sec)
		{
			return this.Context.GetData("FisherTransform", new string[]
			{
				base.get_Period().ToString(),
				this.TriggerLine.ToString(),
				sec.get_CacheName()
			}, () => FisherTransform.GenFisherTransform(sec, this.get_Period(), this.TriggerLine, this.Context));
		}

		// Token: 0x0600017E RID: 382 RVA: 0x000070D4 File Offset: 0x000052D4
		public static IList<double> GenFisherTransform(ISecurity _sec, int _period, bool _TriggerLine, IContext ctx)
		{
			double num = 0.0;
			IList<double> list = new List<double>(_sec.get_Bars().Count);
			IList<double> list2 = new List<double>(_sec.get_Bars().Count);
			IList<double> mids = ctx.GetData("MedianPrice", new string[]
			{
				_sec.get_CacheName()
			}, () => Series.MedianPrice(_sec.get_Bars()));
			IList<double> data = ctx.GetData("medianprice_hhv", new string[]
			{
				_period.ToString(),
				mids.GetHashCode().ToString()
			}, () => Series.Highest(mids, _period));
			IList<double> data2 = ctx.GetData("medianprice_llv", new string[]
			{
				_period.ToString(),
				mids.GetHashCode().ToString()
			}, () => Series.Lowest(mids, _period));
			for (int i = 0; i < _period; i++)
			{
				list.Add(0.0);
				list2.Add(0.0);
			}
			for (int j = _period; j < _sec.get_Bars().Count; j++)
			{
				double num2 = (_sec.get_HighPrices()[j] + _sec.get_LowPrices()[j]) / 2.0;
				double num3 = data[j];
				double num4 = data2[j];
				num = 1.0 * ((num2 - num4) / (num3 - num4) - 0.5) + 0.5 * num;
				num = Math.Min(Math.Max(num, -0.999), 0.999);
				double item = 0.25 * Math.Log((1.0 + num) / (1.0 - num)) + 0.5 * list[j - 1];
				list.Add(item);
				list2.Add(list[j - 1]);
			}
			if (!_TriggerLine)
			{
				return list;
			}
			return list2;
		}

		// Token: 0x1700007F RID: 127
		public IContext Context
		{
			// Token: 0x06000180 RID: 384 RVA: 0x00007401 File Offset: 0x00005601
			get;
			// Token: 0x06000181 RID: 385 RVA: 0x00007409 File Offset: 0x00005609
			set;
		}

		// Token: 0x1700007E RID: 126
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool TriggerLine
		{
			// Token: 0x0600017C RID: 380 RVA: 0x00007081 File Offset: 0x00005281
			get;
			// Token: 0x0600017D RID: 381 RVA: 0x00007089 File Offset: 0x00005289
			set;
		}
	}
}
