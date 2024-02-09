using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000197 RID: 407
	[HandlerCategory("vvAverages"), HandlerName("DSMA")]
	public class DSMA : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000CE6 RID: 3302 RVA: 0x00038C44 File Offset: 0x00036E44
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("dsma", new string[]
			{
				this.DsmaPeriod.ToString(),
				src.GetHashCode().ToString()
			}, () => DSMA.GenDSMA(src, this.DsmaPeriod, this.Context));
		}

		// Token: 0x06000CE5 RID: 3301 RVA: 0x00038AFC File Offset: 0x00036CFC
		public static IList<double> GenDSMA(IList<double> src, int period, IContext ctx)
		{
			IList<double> sma1 = ctx.GetData("sma", new string[]
			{
				period.ToString(),
				src.GetHashCode().ToString()
			}, () => SMA.GenSMA(src, period));
			IList<double> data = ctx.GetData("sma", new string[]
			{
				period.ToString(),
				sma1.GetHashCode().ToString()
			}, () => SMA.GenSMA(sma1, period));
			double[] array = new double[src.Count];
			for (int i = 0; i < src.Count; i++)
			{
				if (i <= period)
				{
					array[i] = src[i];
				}
				else
				{
					array[i] = 2.0 * sma1[i] - data[i];
				}
			}
			return array;
		}

		// Token: 0x17000436 RID: 1078
		public IContext Context
		{
			// Token: 0x06000CE7 RID: 3303 RVA: 0x00038CB0 File Offset: 0x00036EB0
			get;
			// Token: 0x06000CE8 RID: 3304 RVA: 0x00038CB8 File Offset: 0x00036EB8
			set;
		}

		// Token: 0x17000435 RID: 1077
		[HandlerParameter(true, "14", Min = "3", Max = "50", Step = "1")]
		public int DsmaPeriod
		{
			// Token: 0x06000CE3 RID: 3299 RVA: 0x00038ABA File Offset: 0x00036CBA
			get;
			// Token: 0x06000CE4 RID: 3300 RVA: 0x00038AC2 File Offset: 0x00036CC2
			set;
		}
	}
}
