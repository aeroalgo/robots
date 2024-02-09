using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000029 RID: 41
	[HandlerCategory("vvIndicators"), HandlerName("Fisher Yur4ik")]
	public class Fisher_Yur4ik : BasePeriodIndicatorHandler, IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000178 RID: 376 RVA: 0x00007004 File Offset: 0x00005204
		public IList<double> Execute(ISecurity sec)
		{
			return this.Context.GetData("Fisher_Yur4ik", new string[]
			{
				base.get_Period().ToString(),
				sec.get_CacheName()
			}, () => Fisher_Yur4ik.GenFisher_Yur4ik(sec, this.get_Period(), this.Context));
		}

		// Token: 0x06000177 RID: 375 RVA: 0x00006D04 File Offset: 0x00004F04
		public static IList<double> GenFisher_Yur4ik(ISecurity _sec, int _period, IContext context)
		{
			double num = 0.0;
			double num2 = 0.0;
			IList<double> list = new List<double>(_sec.get_Bars().Count);
			IList<double> mids = context.GetData("MedianPrice", new string[]
			{
				_sec.get_CacheName()
			}, () => Series.MedianPrice(_sec.get_Bars()));
			IList<double> data = context.GetData("medianprice_hhv", new string[]
			{
				_period.ToString(),
				mids.GetHashCode().ToString()
			}, () => Series.Highest(mids, _period));
			IList<double> data2 = context.GetData("medianprice_llv", new string[]
			{
				_period.ToString(),
				mids.GetHashCode().ToString()
			}, () => Series.Lowest(mids, _period));
			for (int i = 0; i < _period; i++)
			{
				list.Add(0.0);
			}
			for (int j = _period; j < _sec.get_Bars().Count; j++)
			{
				double num3 = (_sec.get_HighPrices()[j] + _sec.get_LowPrices()[j]) / 2.0;
				double num4 = data[j];
				double num5 = data2[j];
				double num6;
				if (num4 - num5 == 0.0)
				{
					num6 = -0.33 + 0.67 * num;
				}
				else
				{
					num6 = 0.66 * ((num3 - num5) / (num4 - num5) - 0.5) + 0.67 * num;
				}
				num6 = Math.Min(Math.Max(num6, -0.999), 0.999);
				double num7;
				if (1.0 - num6 == 0.0)
				{
					num7 = 0.5 + 0.5 * num2;
				}
				else
				{
					num7 = 0.5 * Math.Log((1.0 + num6) / (1.0 - num6)) + 0.5 * num2;
				}
				num = num6;
				num2 = num7;
				list.Add(num7);
			}
			return list;
		}

		// Token: 0x1700007D RID: 125
		public IContext Context
		{
			// Token: 0x06000179 RID: 377 RVA: 0x00007068 File Offset: 0x00005268
			get;
			// Token: 0x0600017A RID: 378 RVA: 0x00007070 File Offset: 0x00005270
			set;
		}
	}
}
