using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000090 RID: 144
	[HandlerCategory("vvStoch"), HandlerName("Stoch NR")]
	public class StochNR2 : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000510 RID: 1296 RVA: 0x00019BF0 File Offset: 0x00017DF0
		public IList<double> Execute(ISecurity src)
		{
			return this.GenStochNR(src);
		}

		// Token: 0x0600050F RID: 1295 RVA: 0x00019868 File Offset: 0x00017A68
		public IList<double> GenStochNR(ISecurity _sec)
		{
			int num = Math.Max(this.Dperiod, this.Slowing);
			IList<double> Close = _sec.get_ClosePrices();
			IList<double> High = _sec.get_HighPrices();
			IList<double> Low = _sec.get_LowPrices();
			int count = Close.Count;
			double[] array = new double[Close.Count];
			double[] array2 = new double[Close.Count];
			double tick = _sec.get_Tick();
			double num2 = (double)this.NoiseFilter * tick;
			IList<double> data;
			IList<double> data2;
			if (this.UseCloses)
			{
				data = this.Context.GetData("hhv", new string[]
				{
					this.Kperiod.ToString(),
					Close.GetHashCode().ToString()
				}, () => Series.Highest(Close, this.Kperiod));
				data2 = this.Context.GetData("llv", new string[]
				{
					this.Kperiod.ToString(),
					Close.GetHashCode().ToString()
				}, () => Series.Lowest(Close, this.Kperiod));
			}
			else
			{
				data = this.Context.GetData("hhv", new string[]
				{
					this.Kperiod.ToString(),
					_sec.get_CacheName()
				}, () => Series.Highest(High, this.Kperiod));
				data2 = this.Context.GetData("llv", new string[]
				{
					this.Kperiod.ToString(),
					_sec.get_CacheName()
				}, () => Series.Lowest(Low, this.Kperiod));
			}
			for (int i = num; i < count; i++)
			{
				num2 = 0.0;
				double num3 = 0.0;
				double num4 = 0.0;
				double num5 = 0.0;
				for (int j = i - num + 1; j <= i; j++)
				{
					num3 += data[j];
					num4 += data2[j];
					num5 += Close[j];
				}
				num2 *= (double)this.Slowing;
				double num6 = num3 - num4;
				double num7 = num2 - num6;
				if (num7 > 0.0)
				{
					num6 = num2;
					num4 -= num7 / 2.0;
				}
				double num8;
				if (num6 == 0.0)
				{
					num8 = 1.0;
				}
				else
				{
					num8 = 100.0 * (num5 - num4) / num6;
				}
				array[i] = num8;
				int num9 = i - this.Dperiod;
				double num10 = array2[i - 1] * (double)this.Dperiod - array[num9];
				array2[i] = (num10 + array[i]) / (double)this.Dperiod;
			}
			if (this.postSmooth == 0)
			{
				if (!this.SignalLine)
				{
					return array;
				}
				return array2;
			}
			else
			{
				IList<double> result = JMA.GenJMA(array, this.postSmooth, 100);
				IList<double> result2 = JMA.GenJMA(array2, this.postSmooth, 100);
				if (!this.SignalLine)
				{
					return result;
				}
				return result2;
			}
		}

		// Token: 0x170001BC RID: 444
		public IContext Context
		{
			// Token: 0x06000511 RID: 1297 RVA: 0x00019BF9 File Offset: 0x00017DF9
			get;
			// Token: 0x06000512 RID: 1298 RVA: 0x00019C01 File Offset: 0x00017E01
			set;
		}

		// Token: 0x170001B6 RID: 438
		[HandlerParameter(true, "3", Min = "1", Max = "50", Step = "1")]
		public int Dperiod
		{
			// Token: 0x06000503 RID: 1283 RVA: 0x0001979A File Offset: 0x0001799A
			get;
			// Token: 0x06000504 RID: 1284 RVA: 0x000197A2 File Offset: 0x000179A2
			set;
		}

		// Token: 0x170001B5 RID: 437
		[HandlerParameter(true, "10", Min = "1", Max = "30", Step = "1")]
		public int Kperiod
		{
			// Token: 0x06000501 RID: 1281 RVA: 0x00019789 File Offset: 0x00017989
			get;
			// Token: 0x06000502 RID: 1282 RVA: 0x00019791 File Offset: 0x00017991
			set;
		}

		// Token: 0x170001B8 RID: 440
		[HandlerParameter(true, "0", Min = "0", Max = "500", Step = "10")]
		public int NoiseFilter
		{
			// Token: 0x06000507 RID: 1287 RVA: 0x000197BC File Offset: 0x000179BC
			get;
			// Token: 0x06000508 RID: 1288 RVA: 0x000197C4 File Offset: 0x000179C4
			set;
		}

		// Token: 0x170001B9 RID: 441
		[HandlerParameter(true, "2", Min = "1", Max = "15", Step = "1")]
		public int postSmooth
		{
			// Token: 0x06000509 RID: 1289 RVA: 0x000197CD File Offset: 0x000179CD
			get;
			// Token: 0x0600050A RID: 1290 RVA: 0x000197D5 File Offset: 0x000179D5
			set;
		}

		// Token: 0x170001BA RID: 442
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool SignalLine
		{
			// Token: 0x0600050B RID: 1291 RVA: 0x000197DE File Offset: 0x000179DE
			get;
			// Token: 0x0600050C RID: 1292 RVA: 0x000197E6 File Offset: 0x000179E6
			set;
		}

		// Token: 0x170001B7 RID: 439
		[HandlerParameter(true, "3", Min = "1", Max = "50", Step = "1")]
		public int Slowing
		{
			// Token: 0x06000505 RID: 1285 RVA: 0x000197AB File Offset: 0x000179AB
			get;
			// Token: 0x06000506 RID: 1286 RVA: 0x000197B3 File Offset: 0x000179B3
			set;
		}

		// Token: 0x170001BB RID: 443
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool UseCloses
		{
			// Token: 0x0600050D RID: 1293 RVA: 0x000197EF File Offset: 0x000179EF
			get;
			// Token: 0x0600050E RID: 1294 RVA: 0x000197F7 File Offset: 0x000179F7
			set;
		}
	}
}
