using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000019 RID: 25
	[HandlerCategory("vvIndicators"), HandlerName("CMI (Choppy Market Index)")]
	public class CMI : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x060000D2 RID: 210 RVA: 0x00004A94 File Offset: 0x00002C94
		public IList<double> Execute(ISecurity sec)
		{
			return this.Context.GetData("CMI", new string[]
			{
				this.Period.ToString(),
				this.MaPeriod.ToString(),
				this.mamethod.ToString(),
				sec.get_CacheName()
			}, () => CMI.GenCMI(sec, this.Context, this.Period, this.MaPeriod, this.mamethod));
		}

		// Token: 0x060000D1 RID: 209 RVA: 0x000048E0 File Offset: 0x00002AE0
		public static IList<double> GenCMI(ISecurity sec, IContext ctx, int _BarsBack, int _maperiod, int _mamethod)
		{
			int count = sec.get_Bars().Count;
			IList<double> closePrices = sec.get_ClosePrices();
			IList<double> arg_28_0 = sec.get_OpenPrices();
			IList<double> High = sec.get_HighPrices();
			IList<double> Low = sec.get_LowPrices();
			double[] array = new double[count];
			double[] array2 = new double[count];
			IList<double> result = array2;
			IList<double> data = ctx.GetData("hhv", new string[]
			{
				_BarsBack.ToString(),
				sec.get_CacheName()
			}, () => Series.Highest(High, _BarsBack));
			IList<double> data2 = ctx.GetData("llv", new string[]
			{
				_BarsBack.ToString(),
				sec.get_CacheName()
			}, () => Series.Lowest(Low, _BarsBack));
			for (int i = 0; i < count; i++)
			{
				if (i < _BarsBack)
				{
					array[i] = 50.0;
				}
				else
				{
					array[i] = Math.Abs(closePrices[i] - closePrices[i - _BarsBack]) / (data[i] - data2[i]) * 100.0;
				}
				array2[i] = vvSeries.iMA(array, array2, _mamethod, _maperiod, i, 1.0, 1.0);
			}
			return result;
		}

		// Token: 0x17000045 RID: 69
		public IContext Context
		{
			// Token: 0x060000D3 RID: 211 RVA: 0x00004B1B File Offset: 0x00002D1B
			get;
			// Token: 0x060000D4 RID: 212 RVA: 0x00004B23 File Offset: 0x00002D23
			set;
		}

		// Token: 0x17000044 RID: 68
		[HandlerParameter(true, "0", Min = "0", Max = "8", Step = "1", Name = "0-sma,1-ema,3-lwma,4-lrma,5-rema\n6-sinewma,7-zlema,8-hullma")]
		public int mamethod
		{
			// Token: 0x060000CF RID: 207 RVA: 0x0000489E File Offset: 0x00002A9E
			get;
			// Token: 0x060000D0 RID: 208 RVA: 0x000048A6 File Offset: 0x00002AA6
			set;
		}

		// Token: 0x17000043 RID: 67
		[HandlerParameter(true, "10", Min = "3", Max = "15", Step = "1")]
		public int MaPeriod
		{
			// Token: 0x060000CD RID: 205 RVA: 0x0000488D File Offset: 0x00002A8D
			get;
			// Token: 0x060000CE RID: 206 RVA: 0x00004895 File Offset: 0x00002A95
			set;
		}

		// Token: 0x17000042 RID: 66
		[HandlerParameter(true, "60", Min = "30", Max = "90", Step = "5")]
		public int Period
		{
			// Token: 0x060000CB RID: 203 RVA: 0x0000487C File Offset: 0x00002A7C
			get;
			// Token: 0x060000CC RID: 204 RVA: 0x00004884 File Offset: 0x00002A84
			set;
		}
	}
}
