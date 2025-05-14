using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000077 RID: 119
	[HandlerCategory("vvWilliams"), HandlerName("Williams %R")]
	public class WPR : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x0600043C RID: 1084 RVA: 0x00016976 File Offset: 0x00014B76
		public IList<double> Execute(ISecurity sec)
		{
			return WPR.GenWPR(sec, this.WPRPeriod, this.preSmooth, this.postSmooth, this.Context);
		}

		// Token: 0x0600043A RID: 1082 RVA: 0x00016768 File Offset: 0x00014968
		public static IList<double> GenWPR(ISecurity sec, int _period, int _presmooth, int _postsmooth, IContext _ctx)
		{
			int count = sec.get_Bars().Count;
			IList<double> data = _ctx.GetData("hhv", new string[]
			{
				_period.ToString(),
				sec.get_CacheName()
			}, () => Series.Highest(sec.get_HighPrices(), _period));
			IList<double> data2 = _ctx.GetData("llv", new string[]
			{
				_period.ToString(),
				sec.get_CacheName()
			}, () => Series.Lowest(sec.get_LowPrices(), _period));
			IList<double> list = sec.get_ClosePrices();
			if (_presmooth > 0)
			{
				list = JMA.GenJMA(sec.get_ClosePrices(), _presmooth, 100);
			}
			double[] array = new double[count];
			for (int i = 0; i < count; i++)
			{
				if (data[i] - data2[i] != 0.0)
				{
					array[i] = -(data[i] - list[i]) / (data[i] - data2[i]) * 100.0;
				}
				else
				{
					array[i] = -(data[i] - list[i]) / (data[i] - data2[i] + 1E-10) * 100.0;
				}
			}
			IList<double> result = array;
			if (_postsmooth > 0)
			{
				result = JMA.GenJMA(array, _postsmooth, 100);
			}
			return result;
		}

		// Token: 0x0600043B RID: 1083 RVA: 0x00016910 File Offset: 0x00014B10
		public static double iWPR(ISecurity sec, int period, int barNum)
		{
			if (barNum < period)
			{
				period = barNum;
			}
			double result = 0.0;
			double num = vvSeries.iHighest(sec.get_HighPrices(), barNum, period);
			double num2 = vvSeries.iLowest(sec.get_LowPrices(), barNum, period);
			if (num - num2 != 0.0)
			{
				result = -100.0 * (num - sec.get_ClosePrices()[barNum]) / (num - num2);
			}
			return result;
		}

		// Token: 0x17000170 RID: 368
		public IContext Context
		{
			// Token: 0x0600043D RID: 1085 RVA: 0x00016996 File Offset: 0x00014B96
			get;
			// Token: 0x0600043E RID: 1086 RVA: 0x0001699E File Offset: 0x00014B9E
			set;
		}

		// Token: 0x1700016E RID: 366
		[HandlerParameter(true, "0", Min = "3", Max = "50", Step = "1")]
		public int postSmooth
		{
			// Token: 0x06000436 RID: 1078 RVA: 0x0001670B File Offset: 0x0001490B
			get;
			// Token: 0x06000437 RID: 1079 RVA: 0x00016713 File Offset: 0x00014913
			set;
		}

		// Token: 0x1700016F RID: 367
		[HandlerParameter(true, "0", Min = "-100", Max = "100", Step = "25")]
		public int postSmoothPhase
		{
			// Token: 0x06000438 RID: 1080 RVA: 0x0001671C File Offset: 0x0001491C
			get;
			// Token: 0x06000439 RID: 1081 RVA: 0x00016724 File Offset: 0x00014924
			set;
		}

		// Token: 0x1700016D RID: 365
		[HandlerParameter(true, "0", Min = "3", Max = "50", Step = "1")]
		public int preSmooth
		{
			// Token: 0x06000434 RID: 1076 RVA: 0x000166FA File Offset: 0x000148FA
			get;
			// Token: 0x06000435 RID: 1077 RVA: 0x00016702 File Offset: 0x00014902
			set;
		}

		// Token: 0x1700016C RID: 364
		[HandlerParameter(true, "14", Min = "3", Max = "50", Step = "1")]
		public int WPRPeriod
		{
			// Token: 0x06000432 RID: 1074 RVA: 0x000166E9 File Offset: 0x000148E9
			get;
			// Token: 0x06000433 RID: 1075 RVA: 0x000166F1 File Offset: 0x000148F1
			set;
		}
	}
}
