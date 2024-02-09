using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000131 RID: 305
	[HandlerCategory("vvRSI"), HandlerName("RSI (Waddah Attar RSI Def)")]
	public class RSI_Attar_Def : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000905 RID: 2309 RVA: 0x00026107 File Offset: 0x00024307
		public IList<double> Execute(IList<double> src)
		{
			return this.GenRSIdef(src, this.RSIPeriod1, this.RSIPeriod2, this.MA_Type, this.MAdiffSmoothPeriod);
		}

		// Token: 0x06000904 RID: 2308 RVA: 0x00025FA8 File Offset: 0x000241A8
		public IList<double> GenRSIdef(IList<double> _src, int _RSIperiod1, int _RSIperiod2, int _MA_Type, int _MAdiffSmoothPeriod)
		{
			IList<double> list = new List<double>(_src.Count);
			IList<double> result = new List<double>(_src.Count);
			IList<double> data = this.Context.GetData("rsi", new string[]
			{
				_RSIperiod1.ToString(),
				_src.GetHashCode().ToString()
			}, () => Series.RSI(_src, _RSIperiod1));
			IList<double> data2 = this.Context.GetData("rsi", new string[]
			{
				_RSIperiod2.ToString(),
				_src.GetHashCode().ToString()
			}, () => Series.RSI(_src, _RSIperiod2));
			for (int i = 0; i < _src.Count - 1; i++)
			{
				list.Add(data[i] - data2[i]);
			}
			switch (_MA_Type)
			{
			case 0:
				result = Series.SMA(list, _MAdiffSmoothPeriod);
				break;
			case 1:
				result = Series.EMA(list, _MAdiffSmoothPeriod);
				break;
			case 2:
				result = LWMA.GenWMA(list, _MAdiffSmoothPeriod);
				break;
			}
			return result;
		}

		// Token: 0x170002E7 RID: 743
		public IContext Context
		{
			// Token: 0x06000906 RID: 2310 RVA: 0x00026128 File Offset: 0x00024328
			get;
			// Token: 0x06000907 RID: 2311 RVA: 0x00026130 File Offset: 0x00024330
			set;
		}

		// Token: 0x170002E5 RID: 741
		[HandlerParameter(true, "14", Min = "2", Max = "20", Step = "1")]
		public int MAdiffSmoothPeriod
		{
			// Token: 0x06000900 RID: 2304 RVA: 0x00025F55 File Offset: 0x00024155
			get;
			// Token: 0x06000901 RID: 2305 RVA: 0x00025F5D File Offset: 0x0002415D
			set;
		}

		// Token: 0x170002E6 RID: 742
		[HandlerParameter(true, "0", Min = "0", Max = "2", Step = "1")]
		public int MA_Type
		{
			// Token: 0x06000902 RID: 2306 RVA: 0x00025F66 File Offset: 0x00024166
			get;
			// Token: 0x06000903 RID: 2307 RVA: 0x00025F6E File Offset: 0x0002416E
			set;
		}

		// Token: 0x170002E3 RID: 739
		[HandlerParameter(true, "14", Min = "5", Max = "50", Step = "1")]
		public int RSIPeriod1
		{
			// Token: 0x060008FC RID: 2300 RVA: 0x00025F33 File Offset: 0x00024133
			get;
			// Token: 0x060008FD RID: 2301 RVA: 0x00025F3B File Offset: 0x0002413B
			set;
		}

		// Token: 0x170002E4 RID: 740
		[HandlerParameter(true, "28", Min = "5", Max = "50", Step = "1")]
		public int RSIPeriod2
		{
			// Token: 0x060008FE RID: 2302 RVA: 0x00025F44 File Offset: 0x00024144
			get;
			// Token: 0x060008FF RID: 2303 RVA: 0x00025F4C File Offset: 0x0002414C
			set;
		}
	}
}
