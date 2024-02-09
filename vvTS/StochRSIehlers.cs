using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000092 RID: 146
	[HandlerCategory("vvStoch"), HandlerName("Ehlers StochRSI")]
	public class StochRSIehlers : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000528 RID: 1320 RVA: 0x0001A2B4 File Offset: 0x000184B4
		public IList<double> Execute(IList<double> src)
		{
			return this.GenStochRSIehlers(src, this.RSIperiod, this.Stochperiod, this.WMAlength, this.Trigger, this.Context);
		}

		// Token: 0x06000527 RID: 1319 RVA: 0x0001A03C File Offset: 0x0001823C
		public IList<double> GenStochRSIehlers(IList<double> _src, int _RSIperiod, int _Stochperiod, int _WMAlength, bool _trigger, IContext _ctx)
		{
			int count = _src.Count;
			int num = Math.Max(_RSIperiod, _Stochperiod);
			IList<double> rsi = _ctx.GetData("rsi", new string[]
			{
				_RSIperiod.ToString(),
				_src.GetHashCode().ToString()
			}, () => Series.RSI(_src, _RSIperiod));
			IList<double> data = _ctx.GetData("rsillv", new string[]
			{
				_Stochperiod.ToString(),
				rsi.GetHashCode().ToString()
			}, () => Series.Lowest(rsi, _Stochperiod));
			IList<double> data2 = _ctx.GetData("rsihhv", new string[]
			{
				_Stochperiod.ToString(),
				rsi.GetHashCode().ToString()
			}, () => Series.Highest(rsi, _Stochperiod));
			double[] array = new double[count];
			double[] array2 = new double[count];
			double[] Value3 = new double[count];
			for (int i = num; i < count; i++)
			{
				Value3[i] = 0.0;
				if (data2[i] - data[i] != 0.0)
				{
					Value3[i] = 100.0 * ((rsi[i] - data[i]) / (data2[i] - data[i]));
				}
				else
				{
					Value3[i] = 50.0;
				}
			}
			IList<double> data3 = _ctx.GetData("lwma", new string[]
			{
				_WMAlength.ToString(),
				Value3.GetHashCode().ToString()
			}, () => LWMA.GenWMA(Value3, _WMAlength));
			for (int j = 1; j < count; j++)
			{
				array[j] = data3[j];
				array2[j] = array[j - 1];
			}
			if (!_trigger)
			{
				return array;
			}
			return array2;
		}

		// Token: 0x170001C5 RID: 453
		public IContext Context
		{
			// Token: 0x06000529 RID: 1321 RVA: 0x0001A2DB File Offset: 0x000184DB
			get;
			// Token: 0x0600052A RID: 1322 RVA: 0x0001A2E3 File Offset: 0x000184E3
			set;
		}

		// Token: 0x170001C1 RID: 449
		[HandlerParameter(true, "9", Min = "1", Max = "20", Step = "1")]
		public int RSIperiod
		{
			// Token: 0x0600051F RID: 1311 RVA: 0x00019FA1 File Offset: 0x000181A1
			get;
			// Token: 0x06000520 RID: 1312 RVA: 0x00019FA9 File Offset: 0x000181A9
			set;
		}

		// Token: 0x170001C2 RID: 450
		[HandlerParameter(true, "9", Min = "1", Max = "20", Step = "1")]
		public int Stochperiod
		{
			// Token: 0x06000521 RID: 1313 RVA: 0x00019FB2 File Offset: 0x000181B2
			get;
			// Token: 0x06000522 RID: 1314 RVA: 0x00019FBA File Offset: 0x000181BA
			set;
		}

		// Token: 0x170001C4 RID: 452
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool Trigger
		{
			// Token: 0x06000525 RID: 1317 RVA: 0x00019FD4 File Offset: 0x000181D4
			get;
			// Token: 0x06000526 RID: 1318 RVA: 0x00019FDC File Offset: 0x000181DC
			set;
		}

		// Token: 0x170001C3 RID: 451
		[HandlerParameter(true, "9", Min = "1", Max = "20", Step = "1")]
		public int WMAlength
		{
			// Token: 0x06000523 RID: 1315 RVA: 0x00019FC3 File Offset: 0x000181C3
			get;
			// Token: 0x06000524 RID: 1316 RVA: 0x00019FCB File Offset: 0x000181CB
			set;
		}
	}
}
