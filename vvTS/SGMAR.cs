using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000056 RID: 86
	[HandlerCategory("vvIndicators"), HandlerName("SGMAR")]
	public class SGMAR : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x0600030F RID: 783 RVA: 0x00011999 File Offset: 0x0000FB99
		public IList<double> Execute(IList<double> src)
		{
			return this.GenSgmar(src, this.RSIPeriod, this.MAPeriod, this.DrawFastLine);
		}

		// Token: 0x0600030E RID: 782 RVA: 0x0001181C File Offset: 0x0000FA1C
		public IList<double> GenSgmar(IList<double> _source, int _RSIPeriod, int _MAPeriod, bool _DrawFastLine)
		{
			double[] array = new double[_source.Count];
			IList<double> arg_38_0 = _source;
			IList<double> RsiBuf = this.Context.GetData("rsi", new string[]
			{
				_RSIPeriod.ToString(),
				_source.GetHashCode().ToString()
			}, () => Series.RSI(_source, _RSIPeriod));
			IList<double> data = this.Context.GetData("sma", new string[]
			{
				_MAPeriod.ToString(),
				RsiBuf.GetHashCode().ToString()
			}, () => Series.SMA(RsiBuf, _MAPeriod));
			IList<double> data2 = this.Context.GetData("ema", new string[]
			{
				_MAPeriod.ToString(),
				RsiBuf.GetHashCode().ToString()
			}, () => Series.EMA(RsiBuf, _MAPeriod));
			for (int i = 1; i < _source.Count; i++)
			{
				array[i] = data2[i] - data[i];
			}
			if (this.DrawDiffHistogramm)
			{
				return array;
			}
			if (!this.DrawFastLine)
			{
				return data;
			}
			return data2;
		}

		// Token: 0x17000109 RID: 265
		public IContext Context
		{
			// Token: 0x06000310 RID: 784 RVA: 0x000119B4 File Offset: 0x0000FBB4
			get;
			// Token: 0x06000311 RID: 785 RVA: 0x000119BC File Offset: 0x0000FBBC
			set;
		}

		// Token: 0x17000108 RID: 264
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool DrawDiffHistogramm
		{
			// Token: 0x0600030C RID: 780 RVA: 0x000117C7 File Offset: 0x0000F9C7
			get;
			// Token: 0x0600030D RID: 781 RVA: 0x000117CF File Offset: 0x0000F9CF
			set;
		}

		// Token: 0x17000107 RID: 263
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool DrawFastLine
		{
			// Token: 0x0600030A RID: 778 RVA: 0x000117B6 File Offset: 0x0000F9B6
			get;
			// Token: 0x0600030B RID: 779 RVA: 0x000117BE File Offset: 0x0000F9BE
			set;
		}

		// Token: 0x17000105 RID: 261
		[HandlerParameter(true, "8", Min = "5", Max = "25", Step = "1")]
		public int MAPeriod
		{
			// Token: 0x06000306 RID: 774 RVA: 0x00011794 File Offset: 0x0000F994
			get;
			// Token: 0x06000307 RID: 775 RVA: 0x0001179C File Offset: 0x0000F99C
			set;
		}

		// Token: 0x17000106 RID: 262
		[HandlerParameter(true, "14", Min = "6", Max = "25", Step = "1")]
		public int RSIPeriod
		{
			// Token: 0x06000308 RID: 776 RVA: 0x000117A5 File Offset: 0x0000F9A5
			get;
			// Token: 0x06000309 RID: 777 RVA: 0x000117AD File Offset: 0x0000F9AD
			set;
		}
	}
}
