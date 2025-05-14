using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000091 RID: 145
	[HandlerCategory("vvStoch"), HandlerName("StochRSI")]
	public class StochRSI : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x0600051B RID: 1307 RVA: 0x00019EF8 File Offset: 0x000180F8
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("StochRSI", new string[]
			{
				this.RSIperiod.ToString(),
				this.preSmooth.ToString(),
				this.postSmooth.ToString(),
				src.GetHashCode().ToString()
			}, () => StochRSI.GenStochRSI(src, this.Context, this.RSIperiod, this.preSmooth, this.postSmooth));
		}

		// Token: 0x0600051A RID: 1306 RVA: 0x00019C9C File Offset: 0x00017E9C
		public static IList<double> GenStochRSI(IList<double> _src, IContext _ctx, int _RSIperiod, int _preSmooth, int _postSmooth)
		{
			IList<double> price = _src;
			if (_preSmooth >= 1)
			{
				price = _ctx.GetData("jma", new string[]
				{
					_preSmooth.ToString(),
					_src.GetHashCode().ToString()
				}, () => JMA.GenJMA(_src, _preSmooth, 0));
			}
			IList<double> rsi = _ctx.GetData("rsi", new string[]
			{
				_RSIperiod.ToString(),
				price.GetHashCode().ToString()
			}, () => Series.RSI(price, _RSIperiod));
			IList<double> data = _ctx.GetData("rsillv", new string[]
			{
				_RSIperiod.ToString(),
				rsi.GetHashCode().ToString()
			}, () => Series.Lowest(rsi, _RSIperiod));
			IList<double> data2 = _ctx.GetData("rsihhv", new string[]
			{
				_RSIperiod.ToString(),
				rsi.GetHashCode().ToString()
			}, () => Series.Highest(rsi, _RSIperiod));
			double[] array = new double[rsi.Count];
			for (int i = 0; i < rsi.Count; i++)
			{
				double num = data2[i] - data[i];
				array[i] = ((num == 0.0) ? 0.0 : (100.0 * (rsi[i] - data[i]) / (data2[i] - data[i])));
			}
			IList<double> result = array;
			if (_postSmooth >= 1)
			{
				result = JMA.GenJMA(array, _postSmooth, 100);
			}
			return result;
		}

		// Token: 0x170001C0 RID: 448
		public IContext Context
		{
			// Token: 0x0600051C RID: 1308 RVA: 0x00019F88 File Offset: 0x00018188
			get;
			// Token: 0x0600051D RID: 1309 RVA: 0x00019F90 File Offset: 0x00018190
			set;
		}

		// Token: 0x170001BF RID: 447
		[HandlerParameter(true, "0", Min = "1", Max = "20", Step = "1")]
		public int postSmooth
		{
			// Token: 0x06000518 RID: 1304 RVA: 0x00019C34 File Offset: 0x00017E34
			get;
			// Token: 0x06000519 RID: 1305 RVA: 0x00019C3C File Offset: 0x00017E3C
			set;
		}

		// Token: 0x170001BE RID: 446
		[HandlerParameter(true, "0", Min = "1", Max = "20", Step = "1")]
		public int preSmooth
		{
			// Token: 0x06000516 RID: 1302 RVA: 0x00019C23 File Offset: 0x00017E23
			get;
			// Token: 0x06000517 RID: 1303 RVA: 0x00019C2B File Offset: 0x00017E2B
			set;
		}

		// Token: 0x170001BD RID: 445
		[HandlerParameter(true, "9", Min = "1", Max = "20", Step = "1")]
		public int RSIperiod
		{
			// Token: 0x06000514 RID: 1300 RVA: 0x00019C12 File Offset: 0x00017E12
			get;
			// Token: 0x06000515 RID: 1301 RVA: 0x00019C1A File Offset: 0x00017E1A
			set;
		}
	}
}
